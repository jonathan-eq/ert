pub struct EnsembleStarted;
pub struct EnsembleSucceeded;
pub struct EnsembleFailed;
pub struct EnsembleCancelled;

pub enum EnsembleEvent {
    EnsembleStarted(EnsembleStarted),
    EnsembleSucceeded(EnsembleSucceeded),
    EnsembleFailed(EnsembleFailed),
    EnsembleCancelled(EnsembleCancelled),
}

impl EnsembleEvent {
    pub fn get_status(&self) -> &'static str {
        match self {
            EnsembleEvent::EnsembleStarted(_) => "Starting",
            EnsembleEvent::EnsembleSucceeded(_) => "Stopped",
            EnsembleEvent::EnsembleFailed(_) => "Cancelled",
            EnsembleEvent::EnsembleCancelled(_) => "Failed",
        }
    }
}
