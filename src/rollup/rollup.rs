use crate::accounts::{StateReadExt, StateWriteExt};
use crate::rollup::state_ext::StateWriteExt as RollupStateExt;
use crate::snapshot::{self, Snapshot};
use crate::storage::keys::Asset;
use crate::{accounts::StateWriteExt as AccountsStateExt, config::Config};
use crate::{address, execution_service};
use astria_core::execution::v1::Block;
use astria_core::generated::astria::composer::v1::SubmitRollupTransactionRequest;
use astria_core::generated::astria::protocol::transaction::v1::Transaction;
use astria_core::primitive::v1::asset::TracePrefixed;
use astria_core::primitive::v1::RollupId;
use astria_core::protocol::transaction::v1::action::Transfer;
use astria_core::protocol::transaction::v1::TransactionBody;
use astria_core::Protobuf;

use astria_core::generated::astria::composer::v1::grpc_collector_service_client::GrpcCollectorServiceClient;
use astria_core::generated::astria::execution::v1::execution_service_server::ExecutionServiceServer;
use astria_eyre::{anyhow_to_eyre, eyre::WrapErr as _};
use bytes::{buf, Bytes, BytesMut};
use color_eyre::eyre;
use prost::Message;
use rand::rngs::OsRng;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tracing::info;
use warp::Filter;
pub struct Rollup;
use astria_core::crypto::{Signature, SigningKey, VerificationKey};
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
        let mut composer_client = GrpcCollectorServiceClient::connect(composer_addr.clone())
            .await
            .wrap_err("failed to connect to composer")?;
        let alice_secret_bytes: [u8; 32] =
            hex::decode("2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90")
                .unwrap()
                .try_into()
                .unwrap();
        let alice_key = SigningKey::from(alice_secret_bytes);

        let actions = vec![Transfer {
            to: bob_address(),
            amount: 2_000_000_000u128,
            asset: "nria".parse().unwrap(),
            fee_asset: "nria".parse().unwrap(),
        }
        .into()];
        info!("actions: {:?}", actions);
        let transaction_body = TransactionBody::builder()
            .actions(actions)
            .chain_id("test")
            .nonce(1)
            .try_build()
            .unwrap()
            .sign(&alice_key);
        let raw_transaction = transaction_body.to_raw();
        let bytes = raw_transaction.encode_to_vec();
        let send_transfer = composer_client
            .submit_rollup_transaction(SubmitRollupTransactionRequest {
                rollup_id: Some(RollupId::new([69u8; 32]).into_raw()),
                data: bytes.into(),
            })
            .await
            .unwrap();
        info!("send_transfer response: {:?}", send_transfer);
        let signing_key = SigningKey::new(OsRng);

        println!("composer address: {}", composer_addr);
        let storage = cnidarium::Storage::load(cfg.db_filepath.clone(), vec![])
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to load storage backing chain state")?;
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
        println!("address: {:?}", address);
        address.to_prefix("astria").unwrap();
        let asset = crate::accounts::state_ext::nria();
        let balance = 2_000_000_000u128;

        delta
            .put_account_balance(&address, &asset, balance)
            .unwrap();

        delta.put_block(block, 0).unwrap();
        let balance = delta.get_account_balance(&address, &asset).await.unwrap();
        println!("balance: {}", balance);
        let bob_initial_balance = delta
            .get_account_balance(&bob_address(), &asset)
            .await
            .unwrap();
        println!("bob initial balance: {}", bob_initial_balance);

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
        let mut composer_client = GrpcCollectorServiceClient::connect(composer_addr)
            .await
            .wrap_err("failed to connect to composer")?;
        let alice_secret_bytes: [u8; 32] =
            hex::decode("2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90")
                .unwrap()
                .try_into()
                .unwrap();
        let alice_key = SigningKey::from(alice_secret_bytes);

        let actions = vec![Transfer {
            to: bob_address(),
            amount: 333_333,
            asset: "nria".parse().unwrap(),
            fee_asset: "nria".parse().unwrap(),
        }
        .into()];
        info!("actions: {:?}", actions);
        let transaction_body = TransactionBody::builder()
            .actions(actions)
            .chain_id("test")
            .nonce(1)
            .try_build()
            .unwrap()
            .sign(&alice_key);
        let raw_transaction = transaction_body.to_raw();
        let bytes = raw_transaction.encode_to_vec();
        let send_transfer = composer_client
            .submit_rollup_transaction(SubmitRollupTransactionRequest {
                rollup_id: Some(RollupId::new([69u8; 32]).into_raw()),
                data: bytes.into(),
            })
            .await
            .unwrap();
        info!("send_transfer response: {:?}", send_transfer);
        let snapshot = storage.clone().latest_snapshot();
        let state_delta = cnidarium::StateDelta::new(snapshot);
        let bob_final_balance = state_delta
            .get_account_balance(&bob_address(), &asset)
            .await
            .unwrap();
        println!("bob balance: {}", bob_final_balance);

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
// fn with_game_manager(
//     game_manager: game::GameManager,
// ) -> impl Filter<Extract = (game::GameManager,), Error = std::convert::Infallible> + Clone {
//     warp::any().map(move || game_manager.clone())
// }

// // Handler for `POST /create_game/{game_id}`
// async fn handle_create_game(
//     game_id: u32,
//     mut composer_client: GrpcCollectorServiceClient<tonic::transport::channel::Channel>,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     let transaction = Transaction::StartGame { game_id: game_id };
//     println!("encoding transaction: {:?}", transaction);
//     let encoded_transaction = transaction.encode();
//     println!(
//         "submitting transaction to sequencer... encoded transaction {:?}",
//         encoded_transaction
//     );
//     let composer_response = composer_client
//         .submit_rollup_transaction(SubmitRollupTransactionRequest {
//             rollup_id: Some(RollupId {
//                 inner: Bytes::from_static(&[69_u8; 32]),
//             }),
//             data: encoded_transaction,
//         })
//         .await
//         .unwrap();
//     Ok(warp::reply::json(&format!(
//         "Game {} transaction submitted to sequencer",
//         game_id
//     )))
// }

// // Handler for `GET /game_status/{game_id}`
// async fn handle_get_game_status(
//     game_id: u32,
//     game_manager: game::GameManager,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     match game_manager.game_status(game_id) {
//         Ok(status) => {
//             let response = format!("{:?}", status);
//             Ok(warp::reply::json(&response))
//         }
//         Err(_) => Err(warp::reject::not_found()),
//     }
// }
