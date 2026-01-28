use std::time::Instant;

use crate::domain::value_objects::ConnectionState;

/// State transition record
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: ConnectionState,
    pub to: ConnectionState,
    pub timestamp: Instant,
    pub reason: Option<String>,
}

/// Domain entity representing connection lifecycle
#[derive(Debug)]
pub struct ConnectionLifecycle {
    current_state: ConnectionState,
    state_history: Vec<StateTransition>,
    started_at: Option<Instant>,
}

impl ConnectionLifecycle {
    pub fn new() -> Self {
        Self {
            current_state: ConnectionState::Idle,
            state_history: Vec::new(),
            started_at: None,
        }
    }

    pub fn current_state(&self) -> &ConnectionState {
        &self.current_state
    }

    pub fn uptime(&self) -> Option<std::time::Duration> {
        self.started_at.map(|start| start.elapsed())
    }

    pub fn transition_count(&self) -> usize {
        self.state_history.len()
    }

    pub fn last_transition(&self) -> Option<&StateTransition> {
        self.state_history.last()
    }

    /// Transition to connecting state
    pub fn transition_to_connecting(&mut self) {
        self.record_transition(ConnectionState::Connecting, None);
    }

    /// Transition to streaming state
    pub fn transition_to_streaming(&mut self) {
        self.record_transition(ConnectionState::Streaming, None);

        // Track start time when becoming active
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }
    }

    /// Transition to reconnecting state
    pub fn transition_to_reconnecting(&mut self, attempt: u32, reason: Option<String>) {
        self.record_transition(ConnectionState::Reconnecting { attempt }, reason);
    }

    /// Transition to failed state
    pub fn transition_to_failed(&mut self, reason: Option<String>) {
        self.record_transition(ConnectionState::Failed, reason);
    }

    fn record_transition(&mut self, new_state: ConnectionState, reason: Option<String>) {
        let transition = StateTransition {
            from: self.current_state,
            to: new_state,
            timestamp: Instant::now(),
            reason,
        };

        self.state_history.push(transition);
        self.current_state = new_state;
    }

    /// Pure business rule: should we continue retrying?
    pub fn should_continue_retrying(&self, max_failures: Option<u32>) -> bool {
        if let ConnectionState::Reconnecting { attempt } = self.current_state {
            match max_failures {
                Some(max) => attempt < max,
                None => true, // Infinite retries
            }
        } else {
            false
        }
    }
}

impl Default for ConnectionLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_idle() {
        let lifecycle = ConnectionLifecycle::new();
        assert_eq!(*lifecycle.current_state(), ConnectionState::Idle);
        assert_eq!(lifecycle.transition_count(), 0);
    }

    #[test]
    fn test_transitions_are_tracked() {
        let mut lifecycle = ConnectionLifecycle::new();

        lifecycle.transition_to_connecting();
        lifecycle.transition_to_streaming();

        assert_eq!(lifecycle.transition_count(), 2);
        assert_eq!(*lifecycle.current_state(), ConnectionState::Streaming);
    }

    #[test]
    fn test_uptime_tracking() {
        let mut lifecycle = ConnectionLifecycle::new();
        assert!(lifecycle.uptime().is_none());

        lifecycle.transition_to_streaming();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let uptime = lifecycle.uptime().unwrap();
        assert!(uptime.as_millis() >= 10);
    }

    #[test]
    fn test_last_transition() {
        let mut lifecycle = ConnectionLifecycle::new();
        lifecycle.transition_to_connecting();

        let last = lifecycle.last_transition().unwrap();
        assert_eq!(last.from, ConnectionState::Idle);
        assert_eq!(last.to, ConnectionState::Connecting);
    }

    #[test]
    fn test_should_continue_retrying() {
        let mut lifecycle = ConnectionLifecycle::new();
        lifecycle.transition_to_reconnecting(3, Some("test".to_string()));

        // Should retry if under max
        assert!(lifecycle.should_continue_retrying(Some(5)));

        // Should not retry if at max
        assert!(!lifecycle.should_continue_retrying(Some(3)));

        // Should always retry with no max
        assert!(lifecycle.should_continue_retrying(None));
    }
}
