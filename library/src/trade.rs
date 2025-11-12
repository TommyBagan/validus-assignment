use std::{ fmt::Display, marker::PhantomData };

use chrono::{ DateTime, Utc };
use iso_currency::Currency;
use tonic::Status;

use crate::{ error::{ InvalidDetails, UnauthorisedRequester }, state::*, users::* };

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counterparty(pub String);

impl Display for Counterparty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style(pub String);

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    BUY,
    SELL,
}

impl Into<i32> for &Direction {
    fn into(self) -> i32 {
        match self {
            &Direction::BUY => 0,
            &Direction::SELL => 1,
        }
    }
}

impl TryFrom<i32> for Direction {
    type Error = Status;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BUY),
            1 => Ok(Self::SELL),
            _ => { Err(Status::invalid_argument("Direction must either be BUY or SELL")) }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub notional_amount: u64,

    /// A combination of eligible notional currencies.
    /// The notional currency selected must be part of the underlying.
    pub underlying: Vec<Currency>,

    /// The date when the trade value is realized.
    pub value_date: DateTime<Utc>,

    /// The date when the trade assets are delivered.
    pub delivery_date: DateTime<Utc>,
}

#[derive(Debug, Default, Clone)]
pub struct TradeDetailsDiff {
    pub(crate) counterparty: Option<(Counterparty, Counterparty)>,

    pub(crate) direction: Option<(Direction, Direction)>,

    pub(crate) style: Option<(Style, Style)>,

    pub(crate) notional_currency: Option<(Currency, Currency)>,

    pub(crate) notional_amount: Option<(u64, u64)>,

    pub(crate) underlying: Option<(Vec<Currency>, Vec<Currency>)>,

    pub(crate) value_date: Option<(DateTime<Utc>, DateTime<Utc>)>,

    pub(crate) delivery_date: Option<(DateTime<Utc>, DateTime<Utc>)>,

    pub(crate) strike: Option<u64>,
}

impl TradeDetailsDiff {
    pub fn changed_counterparty(&self) -> Option<&(Counterparty, Counterparty)> {
        self.counterparty.as_ref()
    }

    pub fn changed_direction(&self) -> Option<&(Direction, Direction)> {
        self.direction.as_ref()
    }

    pub fn changed_style(&self) -> Option<&(Style, Style)> {
        self.style.as_ref()
    }

    pub fn changed_currency(&self) -> Option<&(Currency, Currency)> {
        self.notional_currency.as_ref()
    }

    pub fn changed_amount(&self) -> Option<(u64, u64)> {
        self.notional_amount
    }

    pub fn changed_underlying(&self) -> Option<&(Vec<Currency>, Vec<Currency>)> {
        self.underlying.as_ref()
    }

    pub fn changed_value_date(&self) -> Option<&(DateTime<Utc>, DateTime<Utc>)> {
        self.value_date.as_ref()
    }

    pub fn changed_delivery_date(&self) -> Option<&(DateTime<Utc>, DateTime<Utc>)> {
        self.delivery_date.as_ref()
    }

    pub fn changed_strike(&self) -> Option<u64> {
        self.strike
    }

    pub(crate) fn new<From: TradeState, To: TradeState>(
        from_details: &TradeDetails<From>,
        to_details: &TradeDetails<To>
    ) -> Option<Self> {
        let from: &MutTradeDetails = &from_details.mutable_details;
        let to: &MutTradeDetails = &to_details.mutable_details;
        if from == to && to_details.strike.is_none() {
            return None;
        }

        let mut diff: Self = Self::default();
        if from.counterparty != to.counterparty {
            diff.counterparty = Some((from.counterparty.clone(), to.counterparty.clone()));
        }
        if from.direction != to.direction {
            diff.direction = Some((from.direction.clone(), to.direction.clone()));
        }
        if from.style != to.style {
            diff.style = Some((from.style.clone(), to.style.clone()));
        }
        if from.notional_currency != to.notional_currency {
            diff.notional_currency = Some((
                from.notional_currency.clone(),
                to.notional_currency.clone(),
            ));
        }
        if from.notional_amount != to.notional_amount {
            diff.notional_amount = Some((from.notional_amount, to.notional_amount));
        }
        if from.underlying != to.underlying {
            diff.underlying = Some((from.underlying.clone(), to.underlying.clone()));
        }
        if from.value_date != to.value_date {
            diff.value_date = Some((from.value_date.clone(), to.value_date.clone()));
        }
        if from.delivery_date != to.delivery_date {
            diff.delivery_date = Some((from.delivery_date.clone(), to.delivery_date.clone()));
        }
        diff.strike = to_details.strike;
        Some(diff)
    }
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
    strike: Option<u64>,

    _state: PhantomData<S>,
}

impl<S: TradeState> TradeDetails<S> {
    pub fn trading_entity(&self) -> &User<Requester> {
        &self.trading_entity
    }

    pub fn counterparty(&self) -> &Counterparty {
        &self.mutable_details.counterparty
    }

    pub fn direction(&self) -> &Direction {
        &self.mutable_details.direction
    }

    pub fn style(&self) -> &Style {
        &self.mutable_details.style
    }

    pub fn currency(&self) -> &Currency {
        &self.mutable_details.notional_currency
    }

    pub fn amount(&self) -> u64 {
        self.mutable_details.notional_amount
    }

    pub fn underlying(&self) -> &Vec<Currency> {
        &self.mutable_details.underlying
    }

    pub fn value_date(&self) -> &DateTime<Utc> {
        &self.mutable_details.value_date
    }

    pub fn delivery_date(&self) -> &DateTime<Utc> {
        &self.mutable_details.delivery_date
    }

    pub fn trade_date(&self) -> &DateTime<Utc> {
        &self.trade_date
    }

    pub fn strike(&self) -> Option<u64> {
        self.strike
    }

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
        amount: u64,
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

impl<S: TradeState> Clone for TradeDetails<S> {
    fn clone(&self) -> Self {
        Self {
            trading_entity: self.trading_entity.clone(),
            mutable_details: self.mutable_details.clone(),
            trade_date: self.trade_date.clone(),
            strike: self.strike.clone(),
            _state: PhantomData,
        }
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
        strike_price: u64,
        user: &U
    ) -> U::TransitionResult<SentToCounterparty, Executed> {
        let mutation = |s: &mut Self| -> () {
            s.strike = Some(strike_price);
        };
        user.transition::<SentToCounterparty, Executed>(self, mutation, TradeAction::Book)
    }
}

#[cfg(test)]
pub(crate) mod tests {
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

    pub(crate) fn mock_draft(requester: &User<Requester>) -> TradeDetails<Draft> {
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
        let wrapped_details: Result<TradeDetails<NeedsReapproval>, _> = details.update(
            &approver,
            new_details
        );
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
