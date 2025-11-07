use crate::chainstate::{
    handle_chainstate_read_block, handle_chainstate_setup, handle_chainstate_teardown,
};
use crate::protocol::{Request, Response, ERR_INTERNAL, ERR_METHOD_NOT_FOUND};
use crate::script_pubkey::handle_script_pubkey_verify;
use crate::state::SessionState;
use std::panic;

pub fn handle_request(req: Request, state: &mut SessionState) -> Response {
    // Catch panics and return error response
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        dispatch_request(req, state)
    }));

    match result {
        Ok(response) => response,
        Err(e) => {
            let panic_msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            Response::error(
                String::new(),
                ERR_INTERNAL,
                format!("Internal error (panic): {}", panic_msg),
            )
        }
    }
}

fn dispatch_request(req: Request, state: &mut SessionState) -> Response {
    match req.method.as_str() {
        // ScriptPubkey
        "script_pubkey.verify" => handle_script_pubkey_verify(req.id, req.params),

        // Chainstate
        "chainstate.setup" => handle_chainstate_setup(req.id, req.params, state),
        "chainstate.read_block" => handle_chainstate_read_block(req.id, req.params, state),
        "chainstate.teardown" => handle_chainstate_teardown(req.id, state),

        _ => Response::error(
            req.id,
            ERR_METHOD_NOT_FOUND,
            format!("Unknown method: {}", req.method),
        ),
    }
}

// Helper for panic::catch_unwind with mutable references
struct AssertUnwindSafe<F>(F);

impl<F> std::ops::Deref for AssertUnwindSafe<F> {
    type Target = F;
    fn deref(&self) -> &F {
        &self.0
    }
}

impl<F> std::ops::DerefMut for AssertUnwindSafe<F> {
    fn deref_mut(&mut self) -> &mut F {
        &mut self.0
    }
}

impl<F: FnOnce() -> R, R> std::panic::UnwindSafe for AssertUnwindSafe<F> {}
