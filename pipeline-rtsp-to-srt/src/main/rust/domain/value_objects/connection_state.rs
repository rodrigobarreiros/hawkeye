use std::fmt;

/// Pipeline connection states (pure domain)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not started yet
    Idle,
    /// Attempting to connect to source
    Connecting,
    /// Successfully streaming data
    Streaming,
    /// Connection lost, will retry
    Reconnecting { attempt: u32 },
    /// Permanent failure or stopped
    Failed,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "IDLE"),
            Self::Connecting => write!(f, "CONNECTING"),
            Self::Streaming => write!(f, "STREAMING"),
            Self::Reconnecting { attempt } => write!(f, "RECONNECTING (attempt {})", attempt),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

impl ConnectionState {
    /// Convert state to numeric value for metrics
    pub fn as_metric(&self) -> f64 {
        match self {
            Self::Idle => 0.0,
            Self::Connecting => 1.0,
            Self::Streaming => 2.0,
            Self::Reconnecting { .. } => 3.0,
            Self::Failed => 4.0,
        }
    }

    /// Check if state is healthy (streaming)
    pub fn is_streaming(&self) -> bool {
        matches!(self, Self::Streaming)
    }

    /// Check if state indicates a problem
    pub fn is_problematic(&self) -> bool {
        matches!(self, Self::Reconnecting { .. } | Self::Failed)
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_idle() {
        assert_eq!(ConnectionState::default(), ConnectionState::Idle);
    }

    #[test]
    fn test_is_streaming() {
        assert!(!ConnectionState::Idle.is_streaming());
        assert!(!ConnectionState::Connecting.is_streaming());
        assert!(ConnectionState::Streaming.is_streaming());
        assert!(!ConnectionState::Reconnecting { attempt: 1 }.is_streaming());
        assert!(!ConnectionState::Failed.is_streaming());
    }

    #[test]
    fn test_is_problematic() {
        assert!(!ConnectionState::Idle.is_problematic());
        assert!(!ConnectionState::Connecting.is_problematic());
        assert!(!ConnectionState::Streaming.is_problematic());
        assert!(ConnectionState::Reconnecting { attempt: 1 }.is_problematic());
        assert!(ConnectionState::Failed.is_problematic());
    }

    #[test]
    fn test_as_metric() {
        assert_eq!(ConnectionState::Idle.as_metric(), 0.0);
        assert_eq!(ConnectionState::Connecting.as_metric(), 1.0);
        assert_eq!(ConnectionState::Streaming.as_metric(), 2.0);
        assert_eq!(ConnectionState::Reconnecting { attempt: 5 }.as_metric(), 3.0);
        assert_eq!(ConnectionState::Failed.as_metric(), 4.0);
    }
}
