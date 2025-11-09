use std::fmt::{Debug, Display};

/// This trait is a marker trait, that acts as our for  
/// our generic for the type state pattern.
/// The type state pattern allows us to enforce that
/// state transitions won't compile if incorrect; and this
/// trait allows us to implement functions which work regardless
/// of the state.
pub trait TradeState: Debug + Display {}

/// For any state which implements this marker trait, the trade its associated with can be cancelled.
pub trait CancellableState: TradeState {}

#[derive(Debug)]
pub struct Draft;

impl Display for Draft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Draft")
    }
}
impl TradeState for Draft {}

#[derive(Debug)]
pub struct PendingApproval;

impl Display for PendingApproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PendingApproval")
    }
}
impl TradeState for PendingApproval {}
impl CancellableState for PendingApproval {}

#[derive(Debug)]
pub struct NeedsReapproval;

impl Display for NeedsReapproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NeedsReapproval")
    }
}
impl TradeState for NeedsReapproval {}
impl CancellableState for NeedsReapproval {}

#[derive(Debug)]
pub struct Approved;

impl Display for Approved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Approved")
    }
}
impl TradeState for Approved {}
impl CancellableState for Approved {}

#[derive(Debug)]
pub struct SentToCounterparty;

impl Display for SentToCounterparty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SentToCounterparty")
    }
}
impl TradeState for SentToCounterparty {}
impl CancellableState for SentToCounterparty {}

#[derive(Debug)]
pub struct Executed;

impl Display for Executed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Executed")
    }
}
impl TradeState for Executed {}

#[derive(Debug)]
pub struct Cancelled;

impl Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cancelled")
    }
}
impl TradeState for Cancelled {}
