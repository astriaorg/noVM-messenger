pub(crate) mod state_ext;
pub(crate) mod storage;
use crate::accounts::{StateReadExt as _, StateWriteExt as _};
use crate::config::Config;
use crate::execution_service;
use crate::rollup::state_ext::StateWriteExt as RollupStateExt;
use crate::snapshot::Snapshot;
use crate::text::{StateReadExt as _, StateWriteExt as _};
use astria_core::crypto::SigningKey;
use astria_core::execution::v1::Block;
use astria_core::generated::astria::composer::v1::grpc_collector_service_client::GrpcCollectorServiceClient;
use astria_core::generated::astria::composer::v1::SubmitRollupTransactionRequest;
use astria_core::generated::astria::execution::v1::execution_service_server::ExecutionServiceServer;
use astria_core::primitive::v1::asset::{self, Denom};
use astria_core::primitive::v1::{Address, RollupId};
use astria_core::Protobuf;
use astria_eyre::{
    anyhow_to_eyre,
    eyre::{Result, WrapErr as _},
};
use bytes::Bytes;
use cnidarium::Storage;
use color_eyre::eyre::{self, eyre};
use prost::Message;
use rollup_core::generated::protocol::transaction::v1::Transaction;
use rollup_core::transaction::v1::action::SendText;
use rollup_core::transaction::v1::{Action, TransactionBody};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use state_ext::StateReadExt;
use std::net::SocketAddr;
use std::str::FromStr;
use thiserror::Error;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tracing::info;
use warp::Filter;

pub struct RollupConfig {
    pub execution_grpc_addr: String,
    pub composer_addr: String,
    pub rollup_name: String,
    pub sequencer_genesis_block_height: u32,
    pub celestia_genesis_block_height: u32,
    pub celestia_block_variance: u64,
}

impl RollupConfig {
    pub fn new(config: Config) -> Self {
        Self {
            execution_grpc_addr: config.execution_grpc_addr,
            composer_addr: config.composer_addr,
            rollup_name: config.rollup_name,
            sequencer_genesis_block_height: config.sequencer_genesis_block_height,
            celestia_genesis_block_height: config.celestia_genesis_block_height,
            celestia_block_variance: config.celestia_block_variance,
        }
    }
}

const CHAIN_ID: &str = "astria-chat";
const FEE_ASSET: &str = "nria";
const FROM: &str = "astria1rsxyjrcm255ds9euthjx6yc3vrjt9sxrm9cfgm";
const SEQUENCER_PRIVATE_KEY: &str =
    "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90";
// const NONCE: u32 = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub sender: String,
    pub message: String,
}

pub struct Rollup;

#[derive(Error, Debug)]
pub enum RestError {
    #[error("Not found")]
    NotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal server error")]
    InternalServerError,
}

impl warp::reject::Reject for RestError {}

impl Rollup {
    pub async fn run_until_stopped(cfg: Config) -> Result<()> {
        let addr: SocketAddr = cfg.execution_grpc_addr.parse()?;
        let composer_addr = cfg.composer_addr.clone();
        let rollup_id = RollupId::from_unhashed_bytes(Sha256::digest(cfg.rollup_name.as_bytes()));
        let warp_rollup_id = warp::any().map(move || rollup_id);

        let composer_client = GrpcCollectorServiceClient::connect(composer_addr.clone())
            .await
            .wrap_err("failed to connect to composer")?;
        let storage = cnidarium::Storage::load(cfg.db_filepath.clone(), vec![])
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to load storage backing chain state")?;

        let submit_transaction = warp::path!("submit_transaction")
            .and(warp::post())
            .and(with_composer(composer_client.clone()))
            .and(warp::body::bytes())
            .and(warp_rollup_id)
            .and_then(handle_submit_transaction);

        let submit_unsigned_message = warp::path!("message")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_composer(composer_client.clone()))
            .and(with_storage(storage.clone()))
            .and(warp_rollup_id)
            .and_then(handle_submit_unsigned_text)
            .with(
                warp::cors()
                    .allow_any_origin()
                    .allow_method(warp::http::Method::POST)
                    .allow_headers(vec!["Content-Type"]),
            );

        let get_account_balance = warp::path!("get_account_balance" / String / String)
            .and(warp::get())
            .and(with_storage(storage.clone()))
            .and_then(handle_get_account_balance);

        let get_account_nonce = warp::path!("get_account_nonce" / String)
            .and(warp::get())
            .and(with_storage(storage.clone()))
            .and_then(handle_get_account_nonce);

        let get_text_from_id = warp::path!("get_text_from_id" / u64)
            .and(warp::get())
            .and(with_storage(storage.clone()))
            .and_then(handle_get_text_from_id);

        let get_recents = warp::path!("recent")
            .and(warp::get())
            .and(with_storage(storage.clone()))
            .and_then(handle_recents)
            .with(warp::cors().allow_any_origin());

        let routes = submit_transaction
            .or(get_account_balance)
            .or(get_text_from_id)
            .or(submit_unsigned_message)
            .or(get_recents)
            .or(get_account_nonce);

        // Spawn the server in a separate async task so it doesn't block the main program
        tokio::spawn(async move {
            warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
        });
        let snapshot_service = Snapshot;
        ServiceBuilder::new()
            .buffer(150)
            .concurrency_limit(3)
            .service(snapshot_service);
        let latest_snapshot = storage.clone().latest_snapshot();
        let mut delta = cnidarium::StateDelta::new(latest_snapshot);

        // Set initial state
        let _block = astria_core::generated::astria::execution::v1::Block {
            number: 0,
            parent_block_hash: Bytes::from_static(&[69u8; 32]),
            hash: Bytes::from_static(&[69u8; 32]),
            timestamp: Some(pbjson_types::Timestamp {
                seconds: 0,
                nanos: 0,
            }),
        };
        let text = "hello world".to_string();
        let address = Address::from_str(FROM)?;
        address.to_prefix("astria")?;
        let asset = crate::accounts::state_ext::nria();
        let balance = 2_000_000_000u128;
        delta.put_account_balance(&address, &asset, balance)?;
        delta.put_text(text, "ido".to_string(), 0).unwrap();
        delta.put_last_text_id(1).unwrap();
        delta
            .put_commitment_state(0, 0, cfg.celestia_genesis_block_height)
            .unwrap();
        if delta.get_block_height().await.is_err() {
            let block = astria_core::generated::astria::execution::v1::Block {
                number: 0,
                parent_block_hash: Bytes::from_static(&[69u8; 32]),
                hash: Bytes::from_static(&[69u8; 32]),
                timestamp: Some(pbjson_types::Timestamp {
                    seconds: 0,
                    nanos: 0,
                }),
            };
            let block = Block::try_from_raw(block).unwrap();
            delta.put_block(block, 0).unwrap();
        }

        let write = storage
            .clone()
            .prepare_commit(delta)
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to prepare commit")?;
        storage
            .clone()
            .commit_batch(write)
            .expect("must be able to successfully commit to storage");
        info!("starting snapshot service server");
        let execution_service = execution_service::RollupExecutionService {
            storage: storage.clone(),
            config: RollupConfig::new(cfg.clone()),
        };

        info!("starting rollup");
        Server::builder()
            .add_service(ExecutionServiceServer::new(execution_service))
            .serve(addr)
            .await?;

        astria_eyre::eyre::Ok(())
    }
}

#[allow(dead_code)]
fn with_composer(
    composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
) -> impl Filter<
    Extract = (GrpcCollectorServiceClient<tonic::transport::channel::Channel>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || composer_client.clone())
}

// Helper function to pass `GameManager` as a filter to endpoints
#[allow(dead_code)]
fn with_storage(
    storage: Storage,
) -> impl Filter<Extract = (Storage,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

async fn handle_submit_transaction(
    mut composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
    data: Bytes,
    rollup_id: RollupId,
) -> Result<impl warp::Reply, warp::Rejection> {
    // let snapshot = storage.latest_snapshot();
    // let delta = cnidarium::StateDelta::new(snapshot);
    info!("received transaction submission request: {:?}", data);
    let raw_transaction = match Transaction::decode(data) {
        Ok(transaction) => transaction,
        Err(_) => {
            return Err(warp::reject::custom(RestError::InvalidInput(
                "failed to decode transaction".to_string(),
            )))
        }
    };

    match composer_client
        .submit_rollup_transaction(SubmitRollupTransactionRequest {
            rollup_id: Some(rollup_id.into_raw()),
            data: raw_transaction.encode_to_vec().into(),
        })
        .await
    {
        Ok(_) => Ok(warp::reply::json(&format!(
            "transaction {:?} submitted to sequencer",
            raw_transaction
        ))),
        Err(_) => Err(warp::reject::reject()),
    }
}

async fn handle_get_account_balance(
    account: String,
    asset: String,
    storage: Storage,
) -> Result<impl warp::Reply, warp::Rejection> {
    let snapshot = storage.latest_snapshot();
    let delta = cnidarium::StateDelta::new(snapshot);
    let denom = Denom::from_str(asset.as_str()).unwrap();
    match delta
        .get_account_balance(&Address::from_str(account.as_str()).unwrap(), &denom)
        .await
    {
        Ok(balance) => Ok(warp::reply::json(&balance.to_string())),
        Err(_) => Err(warp::reject::reject()),
    }
}

async fn handle_get_text_from_id(
    id: u64,
    storage: Storage,
) -> Result<impl warp::Reply, warp::Rejection> {
    let snapshot = storage.latest_snapshot();
    let delta = cnidarium::StateDelta::new(snapshot);
    match delta.get_text(id).await {
        Ok(text) => Ok(warp::reply::json(&String::from(text))),
        Err(_) => Err(warp::reject::reject()),
    }
}

#[allow(dead_code)]
async fn handle_submit_unsigned_text(
    req: SendMessageRequest,
    mut composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
    storage: Storage,
    rollup_id: RollupId,
) -> Result<impl warp::Reply, warp::Rejection> {
    let snapshot = storage.latest_snapshot();
    let delta = cnidarium::StateDelta::new(snapshot);
    let sequencer_key = signing_key_from_private_key(SEQUENCER_PRIVATE_KEY).unwrap();
    let fee_asset = asset::denom::Denom::from_str(FEE_ASSET).unwrap();
    let from_address = address_from_signing_key(&sequencer_key, "astria").unwrap();
    println!("sending tx from address: {from_address}");
    let nonce = match delta.get_account_nonce(&from_address).await {
        Ok(nonce) => Ok(nonce),
        Err(_) => Err(warp::reject::reject()),
    }
    .unwrap();
    let tx = TransactionBody::builder()
        .nonce(nonce)
        .chain_id(CHAIN_ID)
        .actions(vec![Action::Text(SendText {
            text: req.message.clone(),
            from: req.sender.to_string(),
            fee_asset,
        })])
        .try_build()
        .wrap_err("failed to construct a transaction")
        .unwrap()
        .sign(&sequencer_key);
    let raw_encoded: prost::bytes::Bytes = tx.into_raw().encode_to_vec().into();
    composer_client
        .submit_rollup_transaction(SubmitRollupTransactionRequest {
            rollup_id: Some(rollup_id.into_raw()),
            data: raw_encoded,
        })
        .await
        .unwrap();
    Ok(warp::reply::json(&format!(
        "text {:?} submitted to sequencer",
        req.message
    )))
}

#[allow(dead_code)]
async fn handle_recents(storage: Storage) -> Result<impl warp::Reply, warp::Rejection> {
    let snapshot = storage.latest_snapshot();
    let delta = cnidarium::StateDelta::new(snapshot);
    let last_text_id = delta.get_last_text_id().await.unwrap();
    let last_id = u64::from(last_text_id);
    let mut recents = Vec::new();
    for i in 0..last_id {
        let text = String::from(delta.get_text(i).await.unwrap());
        let mut parts = text.splitn(2, ":");
        let sender = parts.next().unwrap_or("").to_string();
        let message = parts.next().unwrap_or("").to_string();
        recents.push(serde_json::json!({
            "sender": sender,
            "message": message,
        }));
    }

    Ok(warp::reply::json(&recents))
}

pub(crate) fn signing_key_from_private_key(private_key: &str) -> eyre::Result<SigningKey> {
    // Decode the hex string to get the private key bytes
    let private_key_bytes: [u8; 32] = hex::decode(private_key)
        .wrap_err("failed to decode private key bytes from hex string")?
        .try_into()
        .map_err(|_| eyre!("invalid private key length; must be 32 bytes"))?;

    // Create and return a signing key from the private key bytes
    Ok(SigningKey::from(private_key_bytes))
}

pub(crate) fn address_from_signing_key(
    signing_key: &SigningKey,
    prefix: &str,
) -> eyre::Result<Address> {
    // Build the address using the public key from the signing key
    let from_address = Address::builder()
        .array(*signing_key.verification_key().address_bytes())
        .prefix(prefix)
        .try_build()
        .wrap_err("failed constructing a valid from address from the provided prefix")?;

    // Return the generated address
    Ok(from_address)
}

async fn handle_get_account_nonce(
    account: String,
    storage: Storage,
) -> Result<impl warp::Reply, warp::Rejection> {
    let snapshot = storage.latest_snapshot();
    let delta = cnidarium::StateDelta::new(snapshot);
    match delta
        .get_account_nonce(&Address::from_str(account.as_str()).unwrap())
        .await
    {
        Ok(nonce) => Ok(warp::reply::json(&nonce.to_string())),
        Err(_) => Err(warp::reject::reject()),
    }
}
