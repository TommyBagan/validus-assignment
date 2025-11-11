use std::{ fmt::{ Debug, Display }, marker::PhantomData };

use crate::{
    error::UnauthorisedRequester,
    state::{ TradeAction, TradeState },
    trade::TradeDetails,
};

pub trait Permission: Debug + PartialEq + Eq {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Requester;

impl Permission for Requester {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Approver;

impl Permission for Approver {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User<P> where P: Permission {
    id: String,
    _permission: PhantomData<P>,
}

impl<P: Permission> Display for User<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<P: Permission> User<P> {
    pub fn sign_in(id: &str) -> Self {
        Self {
            id: id.to_string(),
            _permission: PhantomData,
        }
    }
}

pub trait Transitioner {
    type TransitionResult<From: TradeState, To: TradeState>;

    fn transition<From: TradeState, To: TradeState>(
        &self,
        details: TradeDetails<From>,
        mutation: impl FnOnce(&mut TradeDetails<From>) -> (),
        action: TradeAction
    ) -> Self::TransitionResult<From, To>;
}

impl Transitioner for User<Requester> {
    type TransitionResult<From: TradeState, To: TradeState> = Result<
        TradeDetails<To>,
        UnauthorisedRequester<From>
    >;

    fn transition<From: TradeState, To: TradeState>(
        &self,
        mut details: TradeDetails<From>,
        mutation: impl FnOnce(&mut TradeDetails<From>) -> (),
        action: TradeAction
    ) -> Self::TransitionResult<From, To> {
        if details.trading_entity != *self {
            return Err(UnauthorisedRequester {
                requester: self.id.clone(),
                action: action.to_string(),
                _state: PhantomData
            });
        }
        mutation(&mut details);
        Ok(details.force_transition::<To>())
        // TODO: Log history
    }
}

impl Transitioner for User<Approver> {
    type TransitionResult<From: TradeState, To: TradeState> = TradeDetails<To>;

    fn transition<From: TradeState, To: TradeState>(
        &self,
        mut details: TradeDetails<From>,
        mutation: impl FnOnce(&mut TradeDetails<From>) -> (),
        action: TradeAction
    ) -> Self::TransitionResult<From, To> {
        mutation(&mut details);
        details.force_transition::<To>()
        // TODO: Log history
    }
}
