use std::{ fmt::format, marker::PhantomData };

use chrono::{ DateTime, Utc };
use iso_currency::Currency;

use crate::{ error::{ InvalidDraft, UnauthorisedRequester }, state::*, users::* };

#[derive(Debug)]
pub struct Counterparty(String);

#[derive(Debug)]
pub struct Style(String);

#[derive(Debug)]
pub enum Direction {
    BUY,
    SELL,
}

#[derive(Debug)]
pub struct TradeDetails<S = Draft> where S: TradeState {
    /// Legal entity conducting the trade.
    pub(crate) trading_entity: User<Requester>,

    /// The entity on the other side of the trade.
    counterparty: Counterparty,

    /// Direction of the trade, so buy or sell.
    direction: Direction,

    /// Assumes the trade is a forward contract.
    style: Style,

    /// Currency of the notional amount (e.g., EUR, GBP, USD).
    notional_currency: Currency,

    /// The size of the trade in the selected notional currency.
    notional_amount: u128,

    /// A combination of eligible notional currencies.
    /// The notional currency selected must be part of the underlying.
    underlying: Vec<Currency>,

    /// The date when the trade is initiated.
    trade_date: DateTime<Utc>,

    /// The date when the trade value is realized.
    value_date: DateTime<Utc>,

    /// The date when the trade assets are delivered.
    delivery_date: DateTime<Utc>,

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
            counterparty: self.counterparty,
            direction: self.direction,
            style: self.style,
            notional_currency: self.notional_currency,
            notional_amount: self.notional_amount,
            underlying: self.underlying,
            trade_date: self.trade_date,
            value_date: self.value_date,
            delivery_date: self.delivery_date,
            strike: self.strike,
            _state: PhantomData,
        }
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
    ) -> Result<TradeDetails<Draft>, InvalidDraft> {
        let trade_date = Utc::now();

        if value_date < trade_date || delivery_date < trade_date || delivery_date < value_date {
            return Err(InvalidDraft { issue: "Dates must be chronologically ordered".to_string() });
        }

        if !underlying.contains(&currency) {
            return Err(InvalidDraft {
                issue: format!(
                    "Currency {} not listed in the underlying {}",
                    currency,
                    underlying
                        .into_iter()
                        .map(|c| format!("{},", c.to_string()))
                        .collect::<String>()
                        .trim_end_matches(",")
                ),
            });
        }

        Ok(TradeDetails {
            trading_entity: user.clone(),
            counterparty: counterparty,
            direction: direction,
            style: style,
            notional_currency: currency,
            notional_amount: amount,
            underlying: underlying,
            trade_date: Utc::now(),
            value_date: value_date,
            delivery_date: delivery_date,
            strike: None,
            _state: PhantomData,
        })
    }

    pub fn state() -> &'static str {
        S::NAME
    }
}

impl<S: CancellableState> TradeDetails<S> {
    pub fn cancel<U: Transitioner>(self, user: &U) -> U::TransitionResult<S, Cancelled> {
        user.transition(self, |_| {}, "cancel")
    }
}

impl TradeDetails<Draft> {
    pub fn submit(
        self,
        requester: &User<Requester>
    ) -> Result<TradeDetails<PendingApproval>, UnauthorisedRequester<Draft>> {
        requester.transition::<Draft, PendingApproval>(self, |_| {}, "submit")
    }
}

impl TradeDetails<PendingApproval> {
    pub fn accept(self, approver: &User<Approver>) -> TradeDetails<Approved> {
        approver.transition::<PendingApproval, Approved>(self, |_| {}, "accept")
    }

    pub fn update(
        self,
        approver: &User<Approver>
        //TODO: Add an update field for whatever changes
    ) -> TradeDetails<NeedsReapproval> {
        approver.transition::<PendingApproval, NeedsReapproval>(self, |_| {}, "update")
    }
}

impl TradeDetails<NeedsReapproval> {
    pub fn approve(
        self,
        requester: &User<Requester>
    ) -> Result<TradeDetails<Approved>, UnauthorisedRequester<NeedsReapproval>> {
        requester.transition(self, |_| {}, "approve")
    }
}

impl TradeDetails<Approved> {
    pub fn send_to_execute(self, approver: &User<Approver>) -> TradeDetails<SentToCounterparty> {
        approver.transition::<Approved, SentToCounterparty>(self, |_| {}, "send to execute")
    }
}

impl TradeDetails<SentToCounterparty> {
    pub fn book<U: Transitioner>(
        self,
        strike_price: u128,
        user: &U
    ) -> U::TransitionResult<SentToCounterparty, Executed> {
        let mutation = |s: &mut Self| -> () {
            s.strike = Some(strike_price);
        };
        user.transition::<SentToCounterparty, Executed>(self, mutation, "book")
    }
}

#[cfg(test)]
mod tests {
    use std::{ any::TypeId, time::Duration };

    use chrono::{ FixedOffset, offset };

    use super::*;

    #[test]
    fn bad_drafts() {
        let requester: User<Requester> = User::<Requester>::sign_in("Naughty");
        {
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
            let wrapped_details: Result<TradeDetails, _> = TradeDetails::<Draft>::new(
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
        let details: TradeDetails<NeedsReapproval> = details.update(&approver);

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
}
