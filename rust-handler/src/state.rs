use bitcoinkernel::{ChainstateManager, Context};

pub struct SessionState {
    pub chainstate_manager: Option<ChainstateManager>,
    pub context: Option<Context>,
    pub temp_dir: Option<tempfile::TempDir>,
}

impl SessionState {
    pub fn new() -> Self {
        SessionState {
            chainstate_manager: None,
            context: None,
            temp_dir: None,
        }
    }

    pub fn cleanup(&mut self) {
        // Drop chainstate manager first
        self.chainstate_manager = None;
        // Then drop context
        self.context = None;
        // Finally drop temp dir (which will delete it)
        self.temp_dir = None;
    }
}

impl Drop for SessionState {
    fn drop(&mut self) {
        self.cleanup();
    }
}
