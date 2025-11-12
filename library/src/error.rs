use std::{ error::Error, fmt::{ self, Display }, marker::PhantomData };

use tonic::Status;

use crate::{ state::TradeState };

#[derive(Debug)]
pub struct UnauthorisedRequester<S: TradeState> {
    pub(crate) requester: String,
    pub(crate) action: String,
    pub(crate) _state: PhantomData<S>,
}

impl<S: TradeState> Display for UnauthorisedRequester<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid requester user {} attempted to {} from state {}.",
            self.requester,
            self.action,
            S::NAME
        )
    }
}
impl<S: TradeState> Error for UnauthorisedRequester<S> {}

impl<S: TradeState> Into<Status> for UnauthorisedRequester<S> {
    fn into(self) -> Status {
        Status::unauthenticated(format!("{}", self))
    }
}

#[derive(Debug)]
pub struct InvalidDetails {
    pub(crate) issue: String,
}

impl Display for InvalidDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to create a draft. {}.", self.issue)
    }
}
impl Error for InvalidDetails {}

impl Into<Status> for InvalidDetails {
    fn into(self) -> Status {
        Status::invalid_argument(format!("{}.", self.issue))
    }
}
