use crate::accounts::action::execute_transfer;

#[allow(unused_imports)]
use crate::rollup;
use crate::rollup::state_ext::{StateReadExt, StateWriteExt};
use crate::rollup::RollupConfig;
use crate::text::action::execute_send_text;
use astria_core::execution::v1::Block;

use astria_core::generated::astria::execution::v1::execution_service_server::ExecutionService;
// use astria_core::generated::astria::execution::v1::{self as execution, ExecuteBlockRequest};
use astria_core::generated::astria::execution::v1::{self as execution};
use astria_core::generated::astria::sequencerblock::v1::rollup_data::Value::{
    Deposit, SequencedData,
};
use pbjson_types::Timestamp;
use sha2::{Digest, Sha256};
// use sha2::Digest;

use astria_core::primitive::v1::RollupId;
use astria_core::Protobuf as _;
use bytes::Bytes;
use cnidarium::{StateDelta, Storage};
use prost::Message as _;
// use serde::de;
// use std::os::macos::raw;
use std::sync::Arc;
// use tendermint::node::info;
use tracing::info;

use tonic::{Request, Response, Status};
use tracing::error;

fn compute_block_hash(prev_block_hash: &[u8], merkle_root: &[u8], timestamp: Timestamp) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(prev_block_hash);
    hasher.update(merkle_root);
    hasher.update(timestamp.encode_to_vec());
    hasher.finalize().to_vec()
}

pub(crate) struct RollupExecutionService {
    pub storage: Storage,
    pub config: RollupConfig,
}

#[async_trait::async_trait]
impl ExecutionService for RollupExecutionService {
    async fn get_genesis_info(
        self: Arc<Self>,
        _request: Request<execution::GetGenesisInfoRequest>,
    ) -> Result<Response<execution::GenesisInfo>, Status> {
        let rollup_id =
            RollupId::from_unhashed_bytes(Sha256::digest(self.config.rollup_name.as_bytes()));
        let genesis_info = execution::GenesisInfo {
            rollup_id: Some(rollup_id.into_raw()),
            sequencer_genesis_block_height: self.config.sequencer_genesis_block_height,
            celestia_block_variance: self.config.celestia_block_variance,
        };
        Ok(Response::new(genesis_info))
    }

    async fn get_block(
        self: Arc<Self>,
        request: Request<execution::GetBlockRequest>,
    ) -> Result<Response<execution::Block>, Status> {
        let request = request.into_inner();
        let snapshot = self.storage.latest_snapshot();
        let state_delta = StateDelta::new(snapshot);
        match request.identifier {
            Some(identidfier) => match identidfier.identifier {
                Some(id) => match id {
                    execution::block_identifier::Identifier::BlockNumber(height) => {
                        let block = state_delta.get_block(height).await.unwrap();
                        info!("block: {:?}", block);
                        Ok(Response::new(block.into_raw()))
                    }
                    execution::block_identifier::Identifier::BlockHash(_) => {
                        Err(Status::unimplemented("Get Block by hash not implemented"))
                    }
                },
                None => Err(Status::invalid_argument("missing identifier")),
            },
            None => Err(Status::invalid_argument("missing block identifier")),
        }
    }

    async fn batch_get_blocks(
        self: Arc<Self>,
        request: Request<execution::BatchGetBlocksRequest>,
    ) -> Result<Response<execution::BatchGetBlocksResponse>, Status> {
        let request = request.into_inner();
        let snapshot = self.storage.latest_snapshot();
        let state_delta = StateDelta::new(snapshot);
        let mut blocks = execution::BatchGetBlocksResponse { blocks: Vec::new() };
        for identifier in request.identifiers {
            match identifier.identifier {
                Some(id) => match id {
                    execution::block_identifier::Identifier::BlockNumber(block_number) => {
                        blocks.blocks.push(
                            state_delta
                                .get_block(block_number)
                                .await
                                .unwrap()
                                .to_owned()
                                .into_raw(),
                        );
                    }
                    execution::block_identifier::Identifier::BlockHash(_) => {
                        return Err(Status::unimplemented("Get Block by hash not implemented"))
                    }
                },
                None => return Err(Status::invalid_argument("missing block identifier")),
            }
        }
        Ok(Response::new(blocks))
    }

    async fn execute_block(
        self: Arc<Self>,
        request: Request<execution::ExecuteBlockRequest>,
    ) -> Result<Response<execution::Block>, Status> {
        let request = request.into_inner();
        let timestamp = request.timestamp.unwrap();
        let mut transactions: Vec<Bytes> = Vec::new();
        let mut deposits = Vec::new();

        // collect rollup data
        for rollup_data in request.transactions {
            if let Some(value) = rollup_data.value {
                match value {
                    SequencedData(data) => transactions.push(data),
                    Deposit(data) => deposits.push(data),
                }
            }
        }

        let snapshot = self.storage.latest_snapshot();
        let mut state_delta = StateDelta::new(snapshot);
        let commitment = state_delta.get_commitment_state().await.unwrap();
        let block_height = commitment.soft;

        // Execute transactions
        // let mut executed_deposits: Vec<
        //     astria_core::generated::astria::sequencerblock::v1::Deposit,
        // > = Vec::new();
        let mut executed_transaction = Vec::new();
        for tx in transactions {
            let raw_transaction =
                rollup_core::generated::protocol::transaction::v1::Transaction::decode(tx.clone())
                    .unwrap();

            let transaction =
                rollup_core::transaction::v1::Transaction::try_from_raw(raw_transaction).unwrap();
            let sender = transaction.verification_key().address_bytes();
            let actions = transaction.actions();
            for action in actions {
                match action {
                    rollup_core::transaction::v1::Action::Transfer(transfer) => {
                        execute_transfer(transfer, sender, &mut state_delta)
                            .await
                            .unwrap();
                    }
                    rollup_core::transaction::v1::Action::Text(send_text) => {
                        execute_send_text(send_text, &mut state_delta)
                            .await
                            .unwrap();
                    }
                };
            }
            executed_transaction.push(tx);
        }
        // calculate merkle root of executed transactions
        let mut executed_transactions_merkle = merkle::Tree::new();
        for executed_tx in executed_transaction {
            executed_transactions_merkle.push(executed_tx.as_ref());
        }

        let merkle_root = executed_transactions_merkle.root();
        let block_hash =
            compute_block_hash(&request.prev_block_hash, &merkle_root, timestamp.clone());
        let new_block = astria_core::generated::astria::execution::v1::Block {
            number: block_height + 1,
            parent_block_hash: request.prev_block_hash, // get last block hash
            hash: Bytes::copy_from_slice(&block_hash), // hash with previous block hash and transactions
            timestamp: Some(timestamp),
        };

        let block = Block::try_from_raw(new_block.clone()).unwrap();
        state_delta.put_block(block, block_height + 1).unwrap();
        let write_batch: cnidarium::StagedWriteBatch =
            self.storage.prepare_commit(state_delta).await.unwrap();
        let _hash = self.storage.commit_batch(write_batch).unwrap();
        info!("executed block: {:?}", new_block);

        Ok(Response::new(new_block))
    }

    async fn get_commitment_state(
        self: Arc<Self>,
        _request: Request<execution::GetCommitmentStateRequest>,
    ) -> Result<Response<execution::CommitmentState>, Status> {
        let snapshot = self.storage.latest_snapshot();
        let delta_state = StateDelta::new(snapshot);
        let commitment_state = delta_state.get_commitment_state().await.unwrap();
        let soft = delta_state
            .get_block(commitment_state.soft)
            .await
            .unwrap()
            .into_raw();
        let firm = delta_state
            .get_block(commitment_state.firm)
            .await
            .unwrap()
            .into_raw();
        let celestia_height = commitment_state.celestia;
        Ok(Response::new(execution::CommitmentState {
            soft: Some(soft),
            firm: Some(firm),
            base_celestia_height: celestia_height as u64,
        }))
    }

    async fn update_commitment_state(
        self: Arc<Self>,
        request: Request<execution::UpdateCommitmentStateRequest>,
    ) -> Result<Response<execution::CommitmentState>, Status> {
        let snapshot = self.storage.latest_snapshot();
        let mut state_delta = StateDelta::new(snapshot);
        let commitment_state_request = request.into_inner().commitment_state.unwrap();
        let soft_block_request = commitment_state_request.soft.as_ref().unwrap();
        let firm_block_request = commitment_state_request.firm.as_ref().unwrap();
        let soft_request = soft_block_request.number;
        let firm_request = firm_block_request.number;
        let soft_block = state_delta.get_block(soft_request).await.unwrap();
        let firm_block = state_delta.get_block(firm_request).await.unwrap();
        if *soft_block.hash() != soft_block_request.hash {
            error!(
                "soft block hash does not match: current: {:?},  request: {:?}",
                soft_block.hash().to_owned(),
                soft_block_request.hash
            );
            return Err(Status::invalid_argument("Soft block hash does not match"));
        }
        if *firm_block.hash() != firm_block_request.hash {
            return Err(Status::invalid_argument("Firm block hash does not match"));
        }
        state_delta
            .put_commitment_state(
                soft_request,
                firm_request,
                commitment_state_request.base_celestia_height as u32,
            )
            .unwrap();
        let new_commitment_state = execution::CommitmentState {
            soft: Some(soft_block_request.to_owned()),
            firm: Some(firm_block_request.to_owned()),
            base_celestia_height: commitment_state_request.base_celestia_height,
        };

        let write_batch = self.storage.prepare_commit(state_delta).await.unwrap();
        let _hash = self.storage.commit_batch(write_batch).unwrap();

        Ok(Response::new(new_commitment_state))
    }
}
