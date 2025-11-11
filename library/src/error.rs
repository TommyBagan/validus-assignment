use std::{error::Error, fmt::Display, fmt};

use crate::{state::TradeState, trade::TradeDetails};

#[derive(Debug)]
pub struct UnauthorisedRequester<S: TradeState> {
    pub (crate) details: TradeDetails<S>,
    pub (crate) requester: String,
    pub (crate) action: String,
}

impl<S: TradeState> Display for UnauthorisedRequester<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid requester user {} attempted to {} from state {}.", self.requester, self.action, S::NAME)
    }
}
impl<S: TradeState> Error for UnauthorisedRequester<S> {}

#[derive(Debug)]
pub struct InvalidDraft {
    pub (crate) issue: String
}

impl Display for InvalidDraft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to create a draft. {}.", self.issue)
    }
}
impl Error for InvalidDraft {}
