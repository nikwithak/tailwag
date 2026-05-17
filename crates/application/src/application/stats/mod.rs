#[derive(Debug)]
pub struct Statistics {}

#[derive(Debug)]
pub enum TerminationStatus {
    Panic,
    JobSucceeded,
    JobFailed,
    Terminated,
}

#[derive(Debug)]
pub struct RunResult {
    pub statistics: Statistics,
    pub termination_status: TerminationStatus,
}

impl Default for RunResult {
    fn default() -> Self {
        Self {
            statistics: Statistics {},
            termination_status: TerminationStatus::Terminated,
        }
    }
}
