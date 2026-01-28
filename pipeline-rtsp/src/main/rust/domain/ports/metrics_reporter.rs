use crate::domain::entities::StreamSession;

/// Port for metrics reporting
pub trait MetricsReporter: Send + Sync {
    fn report_session_started(&self, session: &StreamSession);
    fn report_session_stopped(&self, session: &StreamSession);
    fn report_client_connected(&self);
    fn report_client_disconnected(&self);
}
