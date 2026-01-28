use crate::domain::value_objects::ConnectionState;

/// Port for metrics reporting
pub trait MetricsReporter: Send + Sync {
    fn report_state_change(&self, state: &ConnectionState);
    fn report_reconnect_attempt(&self);
    fn report_backoff(&self, delay_secs: f64);
    fn report_srt_state(&self, connected: bool);
    fn report_uptime(&self, uptime_secs: f64);
}
