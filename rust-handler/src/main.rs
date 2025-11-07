mod chainstate;
mod handler;
mod protocol;
mod script_pubkey;
mod state;

use handler::handle_request;
use protocol::{Request, Response, ERR_INVALID_REQUEST};
use state::SessionState;
use std::io::{self, BufRead, Write};

fn main() {
    // Create session state
    let mut state = SessionState::new();

    // Read requests from stdin line by line
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                std::process::exit(1);
            }
        };

        // Parse request
        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::error(
                    String::new(),
                    ERR_INVALID_REQUEST,
                    format!("Failed to parse JSON: {}", e),
                );
                send_response(&mut stdout, &resp);
                continue;
            }
        };

        // Handle request
        let resp = handle_request(req, &mut state);
        send_response(&mut stdout, &resp);
    }
}

fn send_response(stdout: &mut impl Write, resp: &Response) {
    match serde_json::to_string(resp) {
        Ok(json) => {
            if let Err(e) = writeln!(stdout, "{}", json) {
                eprintln!("Error writing response: {}", e);
            }
            if let Err(e) = stdout.flush() {
                eprintln!("Error flushing stdout: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error marshaling response: {}", e);
        }
    }
}
