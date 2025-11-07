use crate::protocol::{Response, ERR_INTERNAL, ERR_INVALID_PARAMS, ERR_KERNEL};
use crate::state::SessionState;
use bitcoinkernel::{Block, ChainType, ChainstateManager, Context};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SetupParams {
    pub chain_type: String,
    pub blocks_hex: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadBlockParams {
    pub height: Option<i32>,
    pub tip: Option<bool>,
}

pub fn handle_chainstate_setup(
    id: String,
    params: serde_json::Value,
    state: &mut SessionState,
) -> Response {
    let params: SetupParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return Response::error(
                id,
                ERR_INVALID_PARAMS,
                format!("Failed to parse params: {}", e),
            );
        }
    };

    // Clean up any existing state
    state.cleanup();

    // Create temp directory
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            return Response::error(
                id,
                ERR_INTERNAL,
                format!("Failed to create temp dir: {}", e),
            );
        }
    };

    let data_dir = temp_dir.path().to_str().unwrap();
    let blocks_dir = temp_dir.path().join("blocks");
    let blocks_dir_str = blocks_dir.to_str().unwrap();

    // Parse chain type
    let chain_type = match params.chain_type.as_str() {
        "mainnet" => ChainType::Mainnet,
        "testnet" => ChainType::Testnet,
        "testnet4" => ChainType::Testnet4,
        "signet" => ChainType::Signet,
        "regtest" => ChainType::Regtest,
        _ => {
            return Response::error(
                id,
                ERR_INVALID_PARAMS,
                format!("Unknown chain type: {}", params.chain_type),
            );
        }
    };

    // Create context
    let context = match Context::builder().chain_type(chain_type).build() {
        Ok(ctx) => ctx,
        Err(e) => {
            return Response::error(id, ERR_KERNEL, format!("Failed to create context: {}", e));
        }
    };

    // Create chainstate manager
    let chainman = match ChainstateManager::builder(&context, data_dir, blocks_dir_str)
        .and_then(|builder| {
            builder
                .worker_threads(1)
                .block_tree_db_in_memory(true)
                .chainstate_db_in_memory(true)
                .wipe_db(true, true)
        })
        .and_then(|builder| builder.build())
    {
        Ok(cm) => cm,
        Err(e) => {
            return Response::error(
                id,
                ERR_KERNEL,
                format!("Failed to create chainstate manager: {}", e),
            );
        }
    };

    // Initialize empty databases
    if let Err(e) = chainman.import_blocks() {
        return Response::error(id, ERR_KERNEL, format!("Failed to initialize: {}", e));
    }

    // Process blocks
    let mut blocks_imported = 0;
    for (i, block_hex) in params.blocks_hex.iter().enumerate() {
        let block_bytes = match hex::decode(block_hex) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Response::error(
                    id,
                    ERR_INVALID_PARAMS,
                    format!("Invalid block hex at index {}: {}", i, e),
                );
            }
        };

        let block = match Block::new(&block_bytes) {
            Ok(b) => b,
            Err(e) => {
                return Response::error(
                    id,
                    ERR_KERNEL,
                    format!("Failed to create block at index {}: {}", i, e),
                );
            }
        };

        let result = chainman.process_block(&block);

        if result.is_rejected() {
            return Response::error(
                id,
                ERR_KERNEL,
                format!("Failed to process block at index {}", i),
            );
        }

        if result.is_new_block() {
            blocks_imported += 1;
        }
    }

    // Get tip height
    let chain = chainman.active_chain();
    let tip_height = chain.height();

    // Store state
    state.context = Some(context);
    state.chainstate_manager = Some(chainman);
    state.temp_dir = Some(temp_dir);

    Response::success(
        id,
        json!({
            "blocks_imported": blocks_imported,
            "tip_height": tip_height,
        }),
    )
}

pub fn handle_chainstate_read_block(
    id: String,
    params: serde_json::Value,
    state: &SessionState,
) -> Response {
    let chainman = match &state.chainstate_manager {
        Some(cm) => cm,
        None => {
            return Response::error(id, ERR_INTERNAL, "Chainstate not initialized".to_string());
        }
    };

    let params: ReadBlockParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return Response::error(
                id,
                ERR_INVALID_PARAMS,
                format!("Failed to parse params: {}", e),
            );
        }
    };

    let chain = chainman.active_chain();

    let block_index = if params.tip.unwrap_or(false) {
        chain.tip()
    } else if let Some(height) = params.height {
        match chain.at_height(height as usize) {
            Some(bi) => bi,
            None => {
                return Response::error(id, ERR_KERNEL, "Block not found".to_string());
            }
        }
    } else {
        return Response::error(
            id,
            ERR_INVALID_PARAMS,
            "Must specify either height or tip".to_string(),
        );
    };

    let height = block_index.height();

    let block = match chainman.read_block_data(&block_index) {
        Ok(b) => b,
        Err(e) => {
            return Response::error(id, ERR_KERNEL, format!("Failed to read block: {}", e));
        }
    };

    let block_bytes = match block.consensus_encode() {
        Ok(bytes) => bytes,
        Err(e) => {
            return Response::error(id, ERR_KERNEL, format!("Failed to serialize block: {}", e));
        }
    };
    let block_hex = hex::encode(block_bytes);

    Response::success(
        id,
        json!({
            "block_hex": block_hex,
            "height": height,
        }),
    )
}

pub fn handle_chainstate_teardown(id: String, state: &mut SessionState) -> Response {
    state.cleanup();
    Response::success(id, json!({"success": true}))
}
