use std::{ fmt::Display, marker::PhantomData };

use chrono::{ DateTime, Utc };
use iso_currency::Currency;

use crate::{ error::{ InvalidDetails, UnauthorisedRequester }, state::*, users::* };

#[derive(Debug, Clone)]
pub struct Counterparty(String);

impl Display for Counterparty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Style(String);

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    BUY,
    SELL,
}

#[derive(Debug, Clone)]
pub struct MutTradeDetails {
    /// The entity on the other side of the trade.
    pub counterparty: Counterparty,

    /// Direction of the trade, so buy or sell.
    pub direction: Direction,

    /// Assumes the trade is a forward contract.
    pub style: Style,

    /// Currency of the notional amount (e.g., EUR, GBP, USD).
    pub notional_currency: Currency,

    /// The size of the trade in the selected notional currency.
    pub notional_amount: u128,

    /// A combination of eligible notional currencies.
    /// The notional currency selected must be part of the underlying.
    pub underlying: Vec<Currency>,

    /// The date when the trade value is realized.
    pub value_date: DateTime<Utc>,

    /// The date when the trade assets are delivered.
    pub delivery_date: DateTime<Utc>,
}

#[derive(Debug)]
pub struct TradeDetails<S = Draft> where S: TradeState {
    /// Legal entity conducting the trade.
    pub(crate) trading_entity: User<Requester>,

    /// All details which can be mutated on update requests.
    mutable_details: MutTradeDetails,

    /// The date when the trade is initiated.
    trade_date: DateTime<Utc>,

    /// Agreed rate. This information is only available after trades are executed.
    strike: Option<u128>,

    _state: PhantomData<S>,
}

impl<S: TradeState> TradeDetails<S> {
    /// This consumes self, creating a new type with the next transition.
    /// It isn't public, as it would allow a transition from any state to another.
    /// Once optimized, this should effectively be a noop.
    pub(crate) fn force_transition<To: TradeState>(self) -> TradeDetails<To> {
        TradeDetails {
            trading_entity: self.trading_entity,
            mutable_details: self.mutable_details.clone(),
            trade_date: self.trade_date,
            strike: self.strike,
            _state: PhantomData,
        }
    }

    /// Common checks that need to be made on every mutation.
    fn check_details(&self, mut_details: &MutTradeDetails) -> Result<(), InvalidDetails> {
        if
            mut_details.value_date < self.trade_date ||
            mut_details.delivery_date < self.trade_date ||
            mut_details.delivery_date < mut_details.value_date
        {
            return Err(InvalidDetails {
                issue: "Dates must be chronologically ordered".to_string(),
            });
        }

        if !mut_details.underlying.contains(&mut_details.notional_currency) {
            return Err(InvalidDetails {
                issue: format!(
                    "Currency {} not listed in the underlying {}",
                    mut_details.notional_currency,
                    mut_details.underlying
                        .clone()
                        .into_iter()
                        .map(|c| format!("{},", c.to_string()))
                        .collect::<String>()
                        .trim_end_matches(",")
                ),
            });
        }

        Ok(())
    }

    pub fn new(
        user: &User<Requester>,
        counterparty: Counterparty,
        direction: Direction,
        style: Style,
        currency: Currency,
        amount: u128,
        underlying: Vec<Currency>,
        value_date: DateTime<Utc>,
        delivery_date: DateTime<Utc>
    ) -> Result<TradeDetails<Draft>, InvalidDetails> {
        let details = TradeDetails {
            trading_entity: user.clone(),
            mutable_details: MutTradeDetails {
                counterparty,
                direction,
                style,
                notional_currency: currency,
                notional_amount: amount,
                underlying,
                value_date,
                delivery_date,
            },
            trade_date: Utc::now(),
            strike: None,
            _state: PhantomData,
        };

        details.check_details(&details.mutable_details)?;

        Ok(details)
    }

    pub fn state() -> &'static str {
        S::NAME
    }
}

impl<S: CancellableState> TradeDetails<S> {
    pub fn cancel<U: Transitioner>(self, user: &U) -> U::TransitionResult<S, Cancelled> {
        user.transition(self, |_| {}, TradeAction::Cancel)
    }
}

impl TradeDetails<Draft> {
    pub fn submit(
        self,
        requester: &User<Requester>
    ) -> Result<TradeDetails<PendingApproval>, UnauthorisedRequester<Draft>> {
        requester.transition::<Draft, PendingApproval>(self, |_| {}, TradeAction::Submit)
    }
}

impl TradeDetails<PendingApproval> {
    pub fn accept(self, approver: &User<Approver>) -> TradeDetails<Approved> {
        approver.transition::<PendingApproval, Approved>(self, |_| {}, TradeAction::Accept)
    }

    pub fn grab_mut_details(&self) -> MutTradeDetails {
        self.mutable_details.clone()
    }

    pub fn update(
        self,
        approver: &User<Approver>,
        new_details: MutTradeDetails
    ) -> Result<TradeDetails<NeedsReapproval>, InvalidDetails> {
        self.check_details(&new_details)?;
        Ok(
            approver.transition::<PendingApproval, NeedsReapproval>(
                self,
                |details| {
                    // TODO: Find difference, append to history
                    details.mutable_details = new_details;
                },
                TradeAction::Update
            )
        )
    }
}

impl TradeDetails<NeedsReapproval> {
    pub fn approve(
        self,
        requester: &User<Requester>
    ) -> Result<TradeDetails<Approved>, UnauthorisedRequester<NeedsReapproval>> {
        requester.transition(self, |_| {}, TradeAction::Approve)
    }
}

impl TradeDetails<Approved> {
    pub fn send_to_execute(self, approver: &User<Approver>) -> TradeDetails<SentToCounterparty> {
        approver.transition::<Approved, SentToCounterparty>(
            self,
            |_| {},
            TradeAction::SendToExecute
        )
    }
}

impl TradeDetails<SentToCounterparty> {
    pub fn book<U: Transitioner>(
        self,
        strike_price: u128,
        user: &U
    ) -> U::TransitionResult<SentToCounterparty, Executed> {
        let mutation = |s: &mut Self| -> () {
            // TODO: Append new strike_price to history
            s.strike = Some(strike_price);
        };
        user.transition::<SentToCounterparty, Executed>(self, mutation, TradeAction::Book)
    }
}

#[cfg(test)]
mod tests {
    use std::{ time::Duration };

    use super::*;

    #[test]
    fn bad_drafts() {
        let requester: User<Requester> = User::<Requester>::sign_in("Naughty");
        {
            let offset: Duration = Duration::from_secs(20);
            let value_date: DateTime<Utc> = Utc::now() + offset;
            let delivery_date: DateTime<Utc> = value_date + offset;
            let wrapped_details: Result<TradeDetails, InvalidDetails> = TradeDetails::<Draft>::new(
                &requester,
                Counterparty("TestCounterParty".to_string()),
                Direction::BUY,
                Style("Some Style".to_string()),
                Currency::GBP,
                100,
                vec![Currency::EUR],
                value_date,
                delivery_date
            );

            assert!(wrapped_details.is_err());
        }
        {
            let offset: Duration = Duration::from_secs(20);
            let value_date: DateTime<Utc> = Utc::now() + offset;
            let delivery_date: DateTime<Utc> = value_date - offset;
            let wrapped_details: Result<TradeDetails, InvalidDetails> = TradeDetails::<Draft>::new(
                &requester,
                Counterparty("TestCounterParty".to_string()),
                Direction::BUY,
                Style("Some Style".to_string()),
                Currency::USD,
                100,
                vec![Currency::USD, Currency::GBP, Currency::EUR],
                value_date,
                delivery_date
            );

            assert!(wrapped_details.is_err());
        }
    }

    fn mock_draft(requester: &User<Requester>) -> TradeDetails<Draft> {
        let offset: Duration = Duration::from_secs(20);
        let value_date: DateTime<Utc> = Utc::now() + offset;
        let delivery_date: DateTime<Utc> = value_date + offset;
        let wrapped_details: Result<TradeDetails, _> = TradeDetails::<Draft>::new(
            &requester,
            Counterparty("TestCounterParty".to_string()),
            Direction::BUY,
            Style("Some Style".to_string()),
            Currency::GBP,
            100,
            vec![Currency::GBP, Currency::EUR],
            value_date,
            delivery_date
        );

        assert!(wrapped_details.is_ok());
        wrapped_details.unwrap()
    }

    #[test]
    fn submitting_and_approving_a_trade() {
        // Draft
        let requester: User<Requester> = User::sign_in("TestUser");
        let details: TradeDetails<Draft> = mock_draft(&requester);

        // Submit
        let wrapped_details: Result<TradeDetails<PendingApproval>, _> = details.submit(&requester);
        assert!(wrapped_details.is_ok());
        let details: TradeDetails<PendingApproval> = wrapped_details.unwrap();

        // Approve
        let approver: User<Approver> = User::<Approver>::sign_in("Admin");
        let _: TradeDetails<Approved> = details.accept(&approver);
    }

    #[test]
    fn updating_a_trade_detail() {
        // Draft
        let requester: User<Requester> = User::sign_in("TestUser");
        let details: TradeDetails<Draft> = mock_draft(&requester);

        // Submit
        let wrapped_details: Result<TradeDetails<PendingApproval>, _> = details.submit(&requester);
        assert!(wrapped_details.is_ok());
        let details: TradeDetails<PendingApproval> = wrapped_details.unwrap();

        // Update
        let approver: User<Approver> = User::sign_in("Admin");
        let mut new_details: MutTradeDetails = details.grab_mut_details();
        new_details.direction = Direction::SELL;
        let wrapped_details: Result<TradeDetails<NeedsReapproval>, _> = details.update(&approver, new_details);
        assert!(wrapped_details.is_ok());
        let details: TradeDetails<NeedsReapproval> = wrapped_details.unwrap();

        // Approve
        let wrapped_details: Result<TradeDetails<Approved>, _> = details.approve(&requester);
        assert!(wrapped_details.is_ok());
    }

    #[test]
    fn approved_trade_sent_to_counterparty() {
        // Draft
        let requester: User<Requester> = User::sign_in("TestUser");
        let details: TradeDetails<Draft> = mock_draft(&requester);

        // Submit
        let wrapped_details: Result<TradeDetails<PendingApproval>, _> = details.submit(&requester);
        assert!(wrapped_details.is_ok());
        let details: TradeDetails<PendingApproval> = wrapped_details.unwrap();

        // Approve
        let approver: User<Approver> = User::sign_in("Admin");
        let details: TradeDetails<Approved> = details.accept(&approver);

        // Send To Execute
        let details: TradeDetails<SentToCounterparty> = details.send_to_execute(&approver);

        // Book
        let wrapped_details: Result<TradeDetails<Executed>, _> = details.book(1000, &requester);
        assert!(wrapped_details.is_ok());
    }

    #[test]
    fn wrong_user() {
        // Draft
        let requester: User<Requester> = User::sign_in("TestUser");
        let details: TradeDetails<Draft> = mock_draft(&requester);

        // Submit
        let malicious: User<Requester> = User::sign_in("MaliciousUser");
        let wrapped_details: Result<
            TradeDetails<PendingApproval>,
            UnauthorisedRequester<Draft>
        > = details.submit(&malicious);
        assert!(wrapped_details.is_err());
    }
}
