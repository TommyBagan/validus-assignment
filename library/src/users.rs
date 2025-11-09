use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use crate::{state::TradeState, trade::TradeDetails};

pub trait Permission: Debug + PartialEq + Eq {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Requester;

impl Permission for Requester {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Approver;

impl Permission for Approver {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User<P>
where
    P: Permission,
{
    id: String,
    _permission: PhantomData<P>,
}

impl<P: Permission> Display for User<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub trait Transitioner {
    type TransitionResult<From: TradeState, To: TradeState>;

    fn transition<From: TradeState, To: TradeState>(
        &self,
        details: TradeDetails<From>,
        mutation: impl Fn(&mut TradeDetails<From>) -> (),
    ) -> Self::TransitionResult<From, To>;
}

impl Transitioner for User<Requester> {
    type TransitionResult<From: TradeState, To: TradeState> =
        Result<TradeDetails<To>, TradeDetails<From>>;

    fn transition<From: TradeState, To: TradeState>(
        &self,
        mut details: TradeDetails<From>,
        mutation: impl Fn(&mut TradeDetails<From>) -> (),
    ) -> Self::TransitionResult<From, To> {
        if details.trading_entity != *self {
            return Err(details);
        }
        mutation(&mut details);
        Ok(details.force_transition::<To>())
    }
}

impl Transitioner for User<Approver> {
    type TransitionResult<From: TradeState, To: TradeState> = TradeDetails<To>;

    fn transition<From: TradeState, To: TradeState>(
        &self,
        mut details: TradeDetails<From>,
        mutation: impl Fn(&mut TradeDetails<From>) -> (),
    ) -> Self::TransitionResult<From, To> {
        mutation(&mut details);
        details.force_transition::<To>()
    }
}

// TODO: Represent in memory storage of all potential users.
// pub struct UserRegistry {}
