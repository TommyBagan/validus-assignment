use std::{ ffi::os_str::Display, sync::{ LazyLock, Mutex } };
use chrono::{ DateTime, Utc };

use crate::{
    state::{ TradeAction, TradeState },
    trade::{ TradeDetails, TradeDetailsDiff },
    users::{ Transitioner, User },
};

/// LazyLock static which is evaluated lazily, meaning: first .lock() will
/// create the initial TradeHistory table.
/// The in-memory history API will rely on this static.
pub static HISTORY: LazyLock<Mutex<TradeHistory>> = LazyLock::new(||
    Mutex::new(TradeHistory::new())
);

#[derive(Debug)]
pub struct TradeHistory {
    records: Vec<HistoricalRecord>,
}

impl TradeHistory {
    pub(crate) fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub(crate) fn add_record(&mut self, record: HistoricalRecord) {
        self.records.push(record);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn total_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_record(&self, step: usize) -> Option<HistoricalRecord> {
        if step >= self.records.len() {
            return None;
        }
        return Some(self.records[step].clone());
    }
}

impl IntoIterator for TradeHistory {
    type Item = HistoricalRecord;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.records.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct HistoricalRecord {
    timestamp: DateTime<Utc>,
    action: TradeAction,
    user_id: String,
    state_before: &'static str,
    state_after: &'static str,
    difference: Option<TradeDetailsDiff>,
}

impl HistoricalRecord {
    pub(crate) fn new<From: TradeState, To: TradeState>(
        action: TradeAction,
        id: String,
        from: &TradeDetails<From>,
        to: &TradeDetails<To>
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            action: action,
            user_id: id,
            state_before: From::NAME,
            state_after: To::NAME,
            difference: TradeDetailsDiff::new(from, to),
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn action(&self) -> &TradeAction {
        &self.action
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn state_before(&self) -> &'static str {
        self.state_before
    }

    pub fn state_after(&self) -> &'static str {
        self.state_after
    }

    pub fn changes(&self) -> Option<&TradeDetailsDiff> {
        self.difference.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ DateTime, Utc };

    use crate::{ history::{ HISTORY, HistoricalRecord }, state::Draft, users::{ Requester, User } };

    #[test]
    fn adding_records_to_lazy_history() {
        let user = User::<Requester>::sign_in("Test123");
        let mut our_history = HISTORY.lock().unwrap();
        our_history.clear();

        assert_eq!(our_history.total_record_count(), 0);

        our_history.add_record(
            HistoricalRecord::new::<Draft, Draft>(
                crate::state::TradeAction::Submit,
                user.to_string(),
                &crate::trade::tests::mock_draft(&user),
                &crate::trade::tests::mock_draft(&user)
            )
        );

        assert_eq!(our_history.total_record_count(), 1);
        assert!(our_history.get_record(0).is_some());
        assert!(our_history.get_record(1).is_none());
    }
}
