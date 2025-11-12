use std::fmt::{ Debug, Display };

/// This trait is a marker trait, that acts as our for
/// our generic for the type state pattern.
/// The type state pattern allows us to enforce that
/// state transitions won't compile if incorrect; and this
/// trait allows us to implement functions which work regardless
/// of the state.
pub trait TradeState: Debug + Display {
    const NAME: &'static str;
    const ID: u8;
}

/// For any state which implements this marker trait, the trade its associated with can be cancelled.
pub trait CancellableState: TradeState {}

#[derive(Debug)]
/// The trade has been created but not submitted.
pub struct Draft;

impl Display for Draft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for Draft {
    const NAME: &'static str = "Draft";
    const ID: u8 = 0;
}

#[derive(Debug)]
/// The trade has been submitted and is awaiting approval.
pub struct PendingApproval;

impl Display for PendingApproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for PendingApproval {
    const NAME: &'static str = "PendingApproval";
    const ID: u8 = 1;
}
impl CancellableState for PendingApproval {}

#[derive(Debug)]
/// The trade details were updated by the approver, requiring
/// reapproval from the original requester.
pub struct NeedsReapproval;

impl Display for NeedsReapproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for NeedsReapproval {
    const NAME: &'static str = "NeedsReapproval";
    const ID: u8 = 2;
}
impl CancellableState for NeedsReapproval {}

#[derive(Debug)]
/// The trade has been approved and is ready to send to the
/// counterparty.
pub struct Approved;

impl Display for Approved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for Approved {
    const NAME: &'static str = "Approved";
    const ID: u8 = 3;
}
impl CancellableState for Approved {}

#[derive(Debug)]
/// The trade has been sent to the counterparty for execution.
pub struct SentToCounterparty;

impl Display for SentToCounterparty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for SentToCounterparty {
    const NAME: &'static str = "SentToCounterparty";
    const ID: u8 = 4;
}
impl CancellableState for SentToCounterparty {}

#[derive(Debug)]
/// The trade has been executed and booked.
pub struct Executed;

impl Display for Executed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for Executed {
    const NAME: &'static str = "Executed";
    const ID: u8 = 5;
}

#[derive(Debug)]
/// The trade has been cancelled.
pub struct Cancelled;

impl Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}
impl TradeState for Cancelled {
    const NAME: &'static str = "Cancelled";
    const ID: u8 = 6;
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
