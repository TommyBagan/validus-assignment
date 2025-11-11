use std::fmt::{ Debug, Display };

/// This trait is a marker trait, that acts as our for
/// our generic for the type state pattern.
/// The type state pattern allows us to enforce that
/// state transitions won't compile if incorrect; and this
/// trait allows us to implement functions which work regardless
/// of the state.
pub trait TradeState: Debug + Display {
    const NAME: &'static str;
}

/// For any state which implements this marker trait, the trade its associated with can be cancelled.
pub trait CancellableState: TradeState {}

#[derive(Debug)]
pub struct Draft;

impl Display for Draft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for Draft {
    const NAME: &'static str = "Draft";
}

#[derive(Debug)]
pub struct PendingApproval;

impl Display for PendingApproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for PendingApproval {
    const NAME: &'static str = "PendingApproval";
}
impl CancellableState for PendingApproval {}

#[derive(Debug)]
pub struct NeedsReapproval;

impl Display for NeedsReapproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for NeedsReapproval {
    const NAME: &'static str = "NeedsReapproval";
}
impl CancellableState for NeedsReapproval {}

#[derive(Debug)]
pub struct Approved;

impl Display for Approved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for Approved {
    const NAME: &'static str = "Approved";
}
impl CancellableState for Approved {}

#[derive(Debug)]
pub struct SentToCounterparty;

impl Display for SentToCounterparty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for SentToCounterparty {
    const NAME: &'static str = "SentToCounterparty";
}
impl CancellableState for SentToCounterparty {}

#[derive(Debug)]
pub struct Executed;

impl Display for Executed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for Executed {
    const NAME: &'static str = "Executed";
}

#[derive(Debug)]
pub struct Cancelled;

impl Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for Cancelled {
    const NAME: &'static str = "Cancelled";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeAction {
    Cancel,
    Submit,
    Accept,
    Update,
    Approve,
    SendToExecute,
    Book,
}

impl ToString for TradeAction {
    fn to_string(&self) -> String {
        let x: &str = match self {
            TradeAction::Cancel => "cancel",
            TradeAction::Submit => "submit",
            TradeAction::Accept => "accept",
            TradeAction::Update => "update",
            TradeAction::Approve => "approve",
            TradeAction::SendToExecute => "send to execute",
            TradeAction::Book => "book",
        };
        x.to_string()
    }
}
