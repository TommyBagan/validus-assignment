use std::{ collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc };

use chrono::{ DateTime, Utc };
use iso_currency::Currency;
use library::{
    error::{ InvalidDetails, UnauthorisedRequester },
    state::{
        Approved,
        Cancelled,
        Draft,
        Executed,
        NeedsReapproval,
        PendingApproval,
        SentToCounterparty,
        TradeState,
    },
    trade::{ Counterparty, Direction, Style, TradeDetails },
    users::{ Requester, User },
};
use proto::{ trade_handler_server::{ TradeHandlerServer, TradeHandler }, TradeUuid };
use tokio::sync::RwLock;
use tonic::{ Response, Status, transport::Server };
use uuid::Uuid;

mod proto {
    tonic::include_proto!("trade");
}

#[derive(Debug, Default)]
/// This is an unfortunate issue with following the
/// generic type state pattern. We've relied on the
/// states being monomorphised via generics, but in
/// these scenarios it'd be best for each type to be
/// dyn-compatible and instead implement some trait.
struct ComposedTradeDetails {
    pending_approval: Option<TradeDetails<PendingApproval>>,
    needs_reapproval: Option<TradeDetails<NeedsReapproval>>,
    approved: Option<TradeDetails<Approved>>,
    sent_to_counterparty: Option<TradeDetails<SentToCounterparty>>,
    executed: Option<TradeDetails<Executed>>,
    cancelled: Option<TradeDetails<Cancelled>>,
}

fn convert_trade_details_to_response<S: TradeState>(
    details: &TradeDetails<S>
) -> Result<proto::TradeStatusResponse, Status> {
    Ok(proto::TradeStatusResponse {
        details: Some(proto::TradeDetails {
            trading_entity: Some(proto::Username { user_id: details.trading_entity().to_string() }),
            subdetails: Some(proto::MutableTradeDetails {
                counterparty: details.counterparty().to_string(),
                direction: <&Direction as Into<i32>>::into(details.direction()),
                style: details.style().to_string(),
                currency_code: details.currency().numeric() as u32,
                currency_amount: details.amount(),
                underlying_currency_codes: details
                    .underlying()
                    .clone()
                    .iter()
                    .map(|c: &Currency| { c.numeric() as u32 })
                    .collect(),
                value_date: details.value_date().to_rfc3339(),
                delivery_date: details.delivery_date().to_rfc3339(),
            }),
            trade_date: details.trade_date().to_rfc3339(),
            strike: details.strike().unwrap_or(0),
        }),
        status: S::ID as i32,
    })
}

#[derive(Default, Debug)]
struct TradeHandlerService {
    /// Would be interested to know if there's a better
    /// whilst still following the generic state pattern.
    mapping: Arc<RwLock<HashMap<Uuid, ComposedTradeDetails>>>,
}

#[tonic::async_trait]
impl TradeHandler for TradeHandlerService {
    async fn status(
        &self,
        request: tonic::Request<proto::TradeStatusRequest>
    ) -> Result<tonic::Response<proto::TradeStatusResponse>, Status> {
        // Sanitisation of inbound request
        let input = request.get_ref();
        let Some(raw_uuid) = &input.uuid else {
            return Err(Status::invalid_argument("UUID not specified"));
        };
        let uuid: Uuid = Uuid::from_str(&raw_uuid.uuid).map_err(|e: uuid::Error| {
            Status::invalid_argument(format!("Invalid UUID, {}.", e))
        })?;

        // Retrieving the details
        let map = self.mapping.read().await;
        let Some(composed) = map.get(&uuid) else {
            return Err(Status::not_found("Trade not found."));
        };
        // Preparing the response
        let response = if let Some(pending_approval) = &composed.pending_approval {
            convert_trade_details_to_response(pending_approval)?
        } else if let Some(needs_reapproval) = &composed.needs_reapproval {
            convert_trade_details_to_response(needs_reapproval)?
        } else if let Some(approved) = &composed.approved {
            convert_trade_details_to_response(approved)?
        } else if let Some(sent_to_counterparty) = &composed.sent_to_counterparty {
            convert_trade_details_to_response(sent_to_counterparty)?
        } else if let Some(executed) = &composed.executed {
            convert_trade_details_to_response(&executed)?
        } else if let Some(cancelled) = &composed.cancelled {
            convert_trade_details_to_response(cancelled)?
        } else {
            return Err(Status::data_loss("Server Error."));
        };
        Ok(Response::<proto::TradeStatusResponse>::new(response))
    }

    async fn submit(
        &self,
        request: tonic::Request<proto::TradeSubmitRequest>
    ) -> Result<tonic::Response<proto::TradeSubmitResponse>, Status> {
        // Sanitisation of the inbound request
        let input = request.get_ref();
        let Some(user) = &input.info else {
            return Err(Status::invalid_argument("Username not specified"));
        };
        let requester = User::<Requester>::sign_in(&user.user_id);

        let Some(raw_details) = &input.details else {
            return Err(Status::invalid_argument("Details not specified"));
        };

        let direction: Direction = raw_details.direction.try_into()?;

        let currency: Currency = Currency::from_numeric(
            raw_details.currency_code
                .try_into()
                .map_err(|_| { Status::invalid_argument("Currency doesn't follow ISO standard.") })?
        ).ok_or(Status::invalid_argument("Currency doesn't follow ISO standard."))?;

        let underlying: Vec<Currency> = raw_details.underlying_currency_codes
            .clone()
            .into_iter()
            .map(|code: u32| {
                Currency::from_numeric(
                    code
                        .try_into()
                        .map_err(|_| {
                            Status::invalid_argument(
                                "Underlying currency codes don't follow ISO standard."
                            )
                        })?
                ).ok_or(
                    Status::invalid_argument("Underlying currency codes don't follow ISO standard.")
                )
            })
            .collect::<Result<Vec<Currency>, Status>>()?;

        let value_date: DateTime<Utc> = raw_details.value_date
            .parse()
            .map_err(|_| {
                Status::invalid_argument("Value Date doesn't follow the UTC standard.")
            })?;

        let delivery_date: DateTime<Utc> = raw_details.delivery_date
            .parse()
            .map_err(|_| {
                Status::invalid_argument("Delivery Date doesn't follow the UTC standard.")
            })?;

        // Creating the draft trade
        let details = TradeDetails::<Draft>
            ::new(
                &requester,
                Counterparty(raw_details.counterparty.clone()),
                direction,
                Style(raw_details.style.clone()),
                currency,
                raw_details.currency_amount,
                underlying,
                value_date,
                delivery_date
            )
            .map_err(<InvalidDetails as Into<Status>>::into)?;

        // Preparing the draft trade for submission
        let details = details
            .submit(&requester)
            .map_err(<UnauthorisedRequester<Draft> as Into<Status>>::into)?;

        let uuid = Uuid::new_v4();

        {
            // Storing the details, scoped to reduce limit write lock.
            let mut map = self.mapping.write().await;
            if map.contains_key(&uuid) {
                return Err(Status::already_exists("Trade has already been submitted."));
            }

            let composed = ComposedTradeDetails {
                pending_approval: Some(details),
                ..ComposedTradeDetails::default()
            };
            (*map).insert(uuid, composed);
        }

        // Sending the response
        Ok(
            Response::<proto::TradeSubmitResponse>::new(proto::TradeSubmitResponse {
                uuid: Some(TradeUuid { uuid: uuid.to_string() }),
            })
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = "[::1]:25565".parse()?;
    println!("TradeHandlerServer listening on {}", address);

    Server::builder()
        .add_service(TradeHandlerServer::new(TradeHandlerService::default()))
        .serve(address).await?;

    Ok(())
}
