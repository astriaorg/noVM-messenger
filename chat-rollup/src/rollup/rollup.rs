use crate::accounts::{StateReadExt as _, StateWriteExt as _};
use crate::config::Config;
use crate::execution_service;
use crate::generated::protocol::transaction::v1::Transaction;
use crate::rollup::state_ext::StateWriteExt as RollupStateExt;
use crate::snapshot::Snapshot;
use crate::text::{StateReadExt as _, StateWriteExt as _};
use astria_core::execution::v1::Block;
use astria_core::generated::astria::composer::v1::grpc_collector_service_client::GrpcCollectorServiceClient;
use astria_core::generated::astria::composer::v1::SubmitRollupTransactionRequest;
use astria_core::generated::astria::execution::v1::execution_service_server::ExecutionServiceServer;
use astria_core::primitive::v1::asset::Denom;
use astria_core::primitive::v1::{Address, RollupId};
use astria_core::Protobuf;
use astria_eyre::{anyhow_to_eyre, eyre::WrapErr as _};
use bytes::Bytes;
use cnidarium::Storage;
use color_eyre::eyre;
use prost::Message;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tracing::info;
use warp::Filter;
pub struct Rollup;
use std::net::SocketAddr;
use std::str::FromStr;
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
        let submit_transaction = warp::path!("submit_transaction")
            .and(warp::post())
            .and(with_composer(composer_client.clone()))
            .and(warp::body::bytes())
            .and_then(handle_submit_transaction);

        let get_account_balance = warp::path!("get_account_balance" / String / String)
            .and(warp::get())
            .and(with_storage(storage.clone()))
            .and_then(handle_get_account_balance);

        let get_text_from_id = warp::path!("get_text_from_id" / u64)
            .and(warp::get())
            .and(with_storage(storage.clone()))
            .and_then(handle_get_text_from_id);

        let routes = submit_transaction
            .or(get_account_balance)
            .or(get_text_from_id);

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
        let text = "itamar why?".to_string();
        let address = Address::from_str("astria1rsxyjrcm255ds9euthjx6yc3vrjt9sxrm9cfgm").unwrap();
        address.to_prefix("astria").unwrap();
        let asset = crate::accounts::state_ext::nria();
        let balance = 2_000_000_000u128;

        delta
            .put_account_balance(&address, &asset, balance)
            .unwrap();

        delta.put_block(block, 0).unwrap();
        delta.put_text(text, 0).unwrap();
        delta.put_last_text_id(0).unwrap();
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

async fn handle_submit_transaction(
    mut composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
    data: Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("received transaction submission request: {:?}", data);
    let raw_transaction: Transaction = Transaction::decode(data).unwrap();
    info!(
        "submitting transaction to sequencer... raw transaction {:?}",
        raw_transaction
    );
    let transaction =
        crate::transaction::v1::Transaction::try_from_raw(raw_transaction.clone()).unwrap();
    let rollup_id = RollupId::new([69_u8; 32]);
    composer_client
        .submit_rollup_transaction(SubmitRollupTransactionRequest {
            rollup_id: Some(rollup_id.into_raw()),
            data: raw_transaction.encode_to_vec().into(),
        })
        .await
        .unwrap();
    Ok(warp::reply::json(&format!(
        "transaction {:?} submitted to sequencer",
        transaction
    )))
}

async fn handle_get_account_balance(
    account: String,
    asset: String,
    storage: Storage,
) -> Result<impl warp::Reply, warp::Rejection> {
    let snapshot = storage.latest_snapshot();
    let delta = cnidarium::StateDelta::new(snapshot);
    let denom = Denom::from_str(asset.as_str()).unwrap();
    let account_balance = delta
        .get_account_balance(&Address::from_str(account.as_str()).unwrap(), &denom)
        .await
        .unwrap();
    let response = account_balance.to_string();
    return Ok(warp::reply::json(&response));

    // Err(_) => Err(warp::reject::not_found()),
}

async fn handle_get_text_from_id(
    id: u64,
    storage: Storage,
) -> Result<impl warp::Reply, warp::Rejection> {
    let snapshot = storage.latest_snapshot();
    let delta = cnidarium::StateDelta::new(snapshot);
    let text = delta.get_text(id).await.unwrap();
    let response = String::from(text);
    return Ok(warp::reply::json(&response));
}
