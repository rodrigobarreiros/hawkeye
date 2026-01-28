use std::time::Instant;
use uuid::Uuid;

use crate::domain::value_objects::{ServerConfig, StreamConfig};

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Starting,
    Active { clients: u32 },
    Stopping,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct StreamSession {
    id: String,
    stream_config: StreamConfig,
    server_config: ServerConfig,
    started_at: Instant,
    state: SessionState,
}

impl StreamSession {
    pub fn new(stream_config: StreamConfig, server_config: ServerConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            stream_config,
            server_config,
            started_at: Instant::now(),
            state: SessionState::Starting,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn stream_config(&self) -> &StreamConfig {
        &self.stream_config
    }

    pub fn server_config(&self) -> &ServerConfig {
        &self.server_config
    }

    pub fn uptime(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn activate(&mut self) {
        self.state = SessionState::Active { clients: 0 };
    }

    pub fn add_client(&mut self) {
        if let SessionState::Active { clients } = &mut self.state {
            *clients += 1;
        }
    }

    pub fn remove_client(&mut self) {
        if let SessionState::Active { clients } = &mut self.state {
            if *clients > 0 {
                *clients -= 1;
            }
        }
    }

    pub fn client_count(&self) -> u32 {
        match &self.state {
            SessionState::Active { clients } => *clients,
            _ => 0,
        }
    }

    pub fn stop(&mut self) {
        self.state = SessionState::Stopping;
    }

    pub fn mark_stopped(&mut self) {
        self.state = SessionState::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_session() -> StreamSession {
        let stream_config = StreamConfig::new(PathBuf::from("/test/video.mp4"));
        let server_config = ServerConfig::new(8554, "/cam1".to_string()).unwrap();
        StreamSession::new(stream_config, server_config)
    }

    #[test]
    fn test_new_session_starts_in_starting_state() {
        let session = create_test_session();
        assert!(matches!(session.state(), SessionState::Starting));
    }

    #[test]
    fn test_activate_changes_to_active_state() {
        let mut session = create_test_session();
        session.activate();
        assert!(matches!(session.state(), SessionState::Active { clients: 0 }));
    }

    #[test]
    fn test_add_client_increments_count() {
        let mut session = create_test_session();
        session.activate();
        session.add_client();
        session.add_client();
        assert_eq!(session.client_count(), 2);
    }

    #[test]
    fn test_remove_client_decrements_count() {
        let mut session = create_test_session();
        session.activate();
        session.add_client();
        session.add_client();
        session.remove_client();
        assert_eq!(session.client_count(), 1);
    }

    #[test]
    fn test_remove_client_does_not_go_negative() {
        let mut session = create_test_session();
        session.activate();
        session.remove_client();
        assert_eq!(session.client_count(), 0);
    }

    #[test]
    fn test_session_has_unique_id() {
        let session1 = create_test_session();
        let session2 = create_test_session();
        assert_ne!(session1.id(), session2.id());
    }
}
