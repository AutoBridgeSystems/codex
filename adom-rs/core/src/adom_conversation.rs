use crate::adom::Adom;
use crate::error::Result as AdomResult;
use crate::protocol::Event;
use crate::protocol::Op;
use crate::protocol::Submission;
use std::path::PathBuf;

pub struct AdomConversation {
    adom: Adom,
    rollout_path: PathBuf,
}

/// Conduit for the bidirectional stream of messages that compose a conversation
/// in Adom.
impl AdomConversation {
    pub(crate) fn new(adom: Adom, rollout_path: PathBuf) -> Self {
        Self {
            adom,
            rollout_path,
        }
    }

    pub async fn submit(&self, op: Op) -> AdomResult<String> {
        self.adom.submit(op).await
    }

    /// Use sparingly: this is intended to be removed soon.
    pub async fn submit_with_id(&self, sub: Submission) -> AdomResult<()> {
        self.adom.submit_with_id(sub).await
    }

    pub async fn next_event(&self) -> AdomResult<Event> {
        self.adom.next_event().await
    }

    pub fn rollout_path(&self) -> PathBuf {
        self.rollout_path.clone()
    }
}
