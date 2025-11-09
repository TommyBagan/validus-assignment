use std::{marker::PhantomData, time::Instant};

use chrono::{DateTime, Utc};
use iso_currency::Currency;

use crate::{state::*, users::*};

#[derive(Debug)]
pub struct Counterparty(String);

// TODO: Assumes forward contract, not too sure what they mean by style.
// I should reach out.
#[derive(Debug)]
pub struct Style(String);

#[derive(Debug)]
pub enum Direction {
    BUY,
    SELL,
}

#[derive(Debug)]
pub struct TradeDetails<S = Draft>
where
    S: TradeState,
{
    /// Legal entity conducting the trade.
    pub(crate) trading_entity: User<Requester>, // TODO: Maybe have type be an Arc? Or just have the user be a clone.

    /// The entity on the other side of the trade.
    pub(crate) counterparty: Counterparty,

    /// Direction of the trade, so buy or sell.
    pub(crate) direction: Direction,

    /// Assumes the trade is a forward contract.
    pub(crate) style: Style,

    /// Currency of the notional amount (e.g., EUR, GBP, USD).
    pub(crate) notional_currency: Currency,

    /// The size of the trade in the selected notional currency.
    pub(crate) notional_amount: u128,

    /// A combination of eligible notional currencies.
    /// The notional currency selected must be part of the underlying.    
    pub(crate) underlying: Vec<Currency>,

    /// The date when the trade is initiated.
    pub(crate) trade_date: DateTime<Utc>,

    /// The date when the trade value is realized.
    pub(crate) value_date: DateTime<Utc>,

    /// The date when the trade assets are delivered.
    pub(crate) delivery_date: DateTime<Utc>,

    /// Agreed rate. This information is only available after trades are executed.
    pub(crate) strike: Option<u128>,

    pub(crate) _state: PhantomData<S>,
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

    pub fn new(user: &User<Requester>,
        counterparty: Counterparty,
        direction: Direction,
        style: Style,
        currency: Currency,
        amount: u128,
        underlying: Vec<Currency>,
        value_date: DateTime<Utc>,
        delivery_date: DateTime<Utc>
    ) -> Result<TradeDetails<Draft>, ()> {
        let trade_date = Utc::now();

        if value_date < trade_date
        || delivery_date < trade_date
        || delivery_date < value_date 
        || !underlying.contains(&currency){
            // TODO: Implement appropriate errors.
            return Err(());
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

    // TODO: Add helper function storing common history for every state transition.
}

impl<S: CancellableState> TradeDetails<S> {
    pub fn cancel<U: Transitioner>(self, user: &U) -> U::TransitionResult<S, Cancelled> {
        user.transition(self, |_| {})
    }
}

impl TradeDetails<Draft> {
    pub fn submit(
        self,
        requester: &User<Requester>,
    ) -> Result<TradeDetails<PendingApproval>, Self> {
        requester.transition::<Draft, PendingApproval>(self, |_| {})
    }
}

impl TradeDetails<PendingApproval> {
    pub fn accept(self, approver: &User<Approver>) -> TradeDetails<Approved> {
        approver.transition::<PendingApproval, Approved>(self, |_| {})
    }

    pub fn update(
        self,
        approver: &User<Approver>,
        //TODO: Add an update field for whatever changes
    ) -> TradeDetails<NeedsReapproval> {
        approver.transition::<PendingApproval, NeedsReapproval>(self, |_| {})
    }
}

impl TradeDetails<NeedsReapproval> {
    pub fn approve(
        self,
        requester: &User<Requester>,
    ) -> Result<TradeDetails<NeedsReapproval>, Self> {
        requester.transition(self, |_| {})
    }
}

impl TradeDetails<Approved> {
    pub fn send_to_execute(self, approver: &User<Approver>) -> TradeDetails<SentToCounterparty> {
        approver.transition::<Approved, SentToCounterparty>(self, |_| {})
    }
}

impl TradeDetails<SentToCounterparty> {
    pub fn book<U: Transitioner>(
        self,
        strike_price: u128,
        user: &U,
    ) -> U::TransitionResult<SentToCounterparty, Executed> {
        let mutation = |s: &mut Self| -> () {
            s.strike = Some(strike_price);
        };
        user.transition::<SentToCounterparty, Executed>(self, mutation)
    }
}
