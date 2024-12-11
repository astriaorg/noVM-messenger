use crate::accounts::{storage, StateReadExt, StateWriteExt};
use crate::config::Config;
use crate::execution_service;
use crate::rollup::state_ext::StateWriteExt as RollupStateExt;
use crate::snapshot::Snapshot;
use crate::storage::keys::Asset;
use astria_core::execution::v1::Block;
use astria_core::generated::astria;
use astria_core::generated::astria::composer::v1::SubmitRollupTransactionRequest;
use astria_core::generated::astria::protocol::transaction::v1::Transaction;
use astria_core::primitive::v1::{AddressBuilder, RollupId};
use astria_core::protocol::transaction::v1::action::Transfer;
use astria_core::protocol::transaction::v1::TransactionBody;
use astria_core::Protobuf;

use astria_core::generated::astria::composer::v1::grpc_collector_service_client::GrpcCollectorServiceClient;
use astria_core::generated::astria::execution::v1::execution_service_server::ExecutionServiceServer;
use astria_eyre::{anyhow_to_eyre, eyre::WrapErr as _};
use bytes::Bytes;
use clap_stdin::FileOrStdin;
use cnidarium::Storage;
use color_eyre::eyre;
use prost::Message;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tracing::info;
use warp::filters::fs::file;
use warp::Filter;
pub struct Rollup;
use astria_core::crypto::SigningKey;
use hex_literal::hex;
use std::net::SocketAddr;
use std::str::FromStr;
const ALICE_ADDRESS_BYTES: [u8; 20] = hex!("1c0c490f1b5528d8173c5de46d131160e4b2c0c3");
const BOB_ADDRESS_BYTES: [u8; 20] = hex!("34fec43c7fcab9aef3b3cf8aba855e41ee69ca3a");
const ASTRIA_ADDRESS_PREFIX: &str = "astria";
fn alice_address() -> astria_core::primitive::v1::Address {
    astria_core::primitive::v1::Address::builder()
        .array(ALICE_ADDRESS_BYTES)
        .prefix(ASTRIA_ADDRESS_PREFIX)
        .try_build()
        .unwrap()
}
fn bob_address() -> astria_core::primitive::v1::Address {
    astria_core::primitive::v1::Address::builder()
        .array(BOB_ADDRESS_BYTES)
        .prefix(ASTRIA_ADDRESS_PREFIX)
        .try_build()
        .unwrap()
}
impl Rollup {
    pub async fn run_until_stopped(cfg: Config) -> eyre::Result<()> {
        let addr: SocketAddr = cfg.grpc_addr.parse()?;
        let composer_addr = cfg.composer_addr;
        let composer_client = GrpcCollectorServiceClient::connect(composer_addr.clone())
            .await
            .wrap_err("failed to connect to composer")?;
        println!("composer address: {}", composer_addr);
        let storage = cnidarium::Storage::load(cfg.db_filepath.clone(), vec![])
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to load storage backing chain state")?;
        let submit_transaction = warp::path!("submit_transaction" / String)
            .and(warp::post())
            .and(with_composer(composer_client.clone()))
            .and_then(handle_submit_transaction);

        let get_account_balance = warp::path!("get_account_balance" / String / String)
            .and(warp::get())
            .and(with_storage(storage.clone()))
            .and_then(handle_get_account_balance);
        let routes = submit_transaction.or(get_account_balance);

        println!("Rest server listening on {}", 3030);
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
        let address = astria_core::primitive::v1::Address::from_str(
            "astria1rsxyjrcm255ds9euthjx6yc3vrjt9sxrm9cfgm",
        )
        .unwrap();
        address.to_prefix("astria").unwrap();
        let asset = crate::accounts::state_ext::nria();
        let balance = 2_000_000_000u128;

        delta
            .put_account_balance(&address, &asset, balance)
            .unwrap();

        delta.put_block(block, 0).unwrap();

        delta.put_commitment_state(0, 0, 2).unwrap();
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
        };
        info!("starting rollup");
        Server::builder()
            .add_service(ExecutionServiceServer::new(execution_service))
            .serve(addr)
            .await?;

        astria_eyre::eyre::Ok(())
    }
}

fn with_composer(
    composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
) -> impl Filter<
    Extract = (GrpcCollectorServiceClient<tonic::transport::channel::Channel>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || composer_client.clone())
}

// // Helper function to pass `GameManager` as a filter to endpoints
fn with_storage(
    storage: Storage,
) -> impl Filter<Extract = (Storage,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

fn read_transaction(
    input: FileOrStdin,
) -> eyre::Result<astria_core::protocol::transaction::v1::Transaction> {
    let wire_body: <astria_core::protocol::transaction::v1::Transaction as Protobuf>::Raw =
        serde_json::from_reader(std::io::BufReader::new(input.into_reader()?)).wrap_err_with(
            || {
                format!(
                    "failed to parse input as json `{}`",
                    astria_core::protocol::transaction::v1::Transaction::full_name()
                )
            },
        )?;
    astria_core::protocol::transaction::v1::Transaction::try_from_raw(wire_body)
        .wrap_err("failed to validate transaction body")
}

// // Handler for `POST /create_game/{game_id}`
async fn handle_submit_transaction(
    path: String,
    mut composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let file_or_stdin = FileOrStdin::from_str(&path).unwrap();
    let file = file_or_stdin.filename();
    let transaction = read_transaction(file_or_stdin.clone())
        .wrap_err_with(|| format!("to signed transaction from `{file}`"))
        .unwrap();
    println!(
        "submitting transaction to sequencer... raw transaction {:?}",
        transaction
    );
    let transaction = astria_core::protocol::transaction::v1::Transaction::try_from_raw(
        transaction.to_raw().clone(),
    )
    .unwrap();
    let composer_response = composer_client
        .submit_rollup_transaction(SubmitRollupTransactionRequest {
            rollup_id: Some(astria::primitive::v1::RollupId {
                inner: Bytes::from_static(&[69_u8; 32]),
            }),
            data: transaction.to_raw().encode_to_vec().into(),
        })
        .await
        .unwrap();
    Ok(warp::reply::json(&format!(
        "transaction {:?} submitted to sequencer",
        transaction
    )))
}

// // Handler for `GET /game_status/{game_id}`
async fn handle_get_account_balance(
    account: String,
    asset: String,
    storage: Storage,
) -> Result<impl warp::Reply, warp::Rejection> {
    let snapshot = storage.latest_snapshot();
    let delta = cnidarium::StateDelta::new(snapshot);
    let denom = astria_core::primitive::v1::asset::Denom::from_str(asset.as_str()).unwrap();
    let account_balance = delta
        .get_account_balance(
            &astria_core::primitive::v1::Address::from_str(account.as_str()).unwrap(),
            &denom,
        )
        .await
        .unwrap();
    let response = account_balance.to_string();
    return Ok(warp::reply::json(&response));

    // Err(_) => Err(warp::reject::not_found()),
}
