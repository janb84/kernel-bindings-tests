use crate::protocol::{Response, ERR_INVALID_PARAMS, ERR_KERNEL, ERR_SCRIPT_VERIFY};
use bitcoinkernel::{verify, KernelError, ScriptPubkey, Transaction, TxOut};
use bitcoinkernel::{VERIFY_ALL, VERIFY_ALL_PRE_TAPROOT, VERIFY_NONE};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct VerifyParams {
    pub script_pubkey_hex: String,
    pub amount: i64,
    pub tx_hex: String,
    pub input_index: usize,
    pub flags: String,
}

pub fn handle_script_pubkey_verify(
    id: String,
    params: serde_json::Value,
) -> Response {
    let params: VerifyParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return Response::error(
                id,
                ERR_INVALID_PARAMS,
                format!("Failed to parse params: {}", e),
            );
        }
    };

    // Decode script pubkey
    let script_bytes = if params.script_pubkey_hex.is_empty() {
        vec![]
    } else {
        match hex::decode(&params.script_pubkey_hex) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Response::error(
                    id,
                    ERR_INVALID_PARAMS,
                    format!("Invalid script pubkey hex: {}", e),
                );
            }
        }
    };

    // Decode transaction
    let tx_bytes = match hex::decode(&params.tx_hex) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Response::error(
                id,
                ERR_INVALID_PARAMS,
                format!("Invalid transaction hex: {}", e),
            );
        }
    };

    // Create script pubkey
    let script_pubkey = match ScriptPubkey::new(&script_bytes) {
        Ok(s) => s,
        Err(e) => {
            return Response::error(
                id,
                ERR_KERNEL,
                format!("Failed to create script pubkey: {}", e),
            );
        }
    };

    // Create transaction
    let tx = match Transaction::new(&tx_bytes) {
        Ok(t) => t,
        Err(e) => {
            return Response::error(
                id,
                ERR_KERNEL,
                format!("Failed to create transaction: {}", e),
            );
        }
    };

    // Parse flags
    let flags = match params.flags.as_str() {
        "VERIFY_ALL_NO_TAPROOT" => VERIFY_ALL_PRE_TAPROOT,
        "VERIFY_ALL" => VERIFY_ALL,
        "VERIFY_NONE" => VERIFY_NONE,
        _ => {
            return Response::error(
                id,
                ERR_INVALID_PARAMS,
                format!("Unknown flags: {}", params.flags),
            );
        }
    };

    // Verify script
    let empty_spent_outputs: Vec<TxOut> = vec![];
    let result = verify(
        &script_pubkey,
        Some(params.amount),
        &tx,
        params.input_index,
        Some(flags),
        &empty_spent_outputs,
    );

    match result {
        Ok(()) => Response::success(id, json!({"valid": true})),
        Err(KernelError::ScriptVerify(_)) => {
            Response::error(id, ERR_SCRIPT_VERIFY, "Script verification failed".to_string())
        }
        Err(e) => Response::error(id, ERR_KERNEL, format!("Verification error: {}", e)),
    }
}
