use crate::accounts::action::execute_transfer;
use crate::rollup::state_ext::{StateReadExt, StateWriteExt};
use crate::text::action::execute_send_text;
use astria_core::execution::v1::Block;

use astria_core::generated::astria::execution::v1::execution_service_server::ExecutionService;
use astria_core::generated::astria::execution::v1::{self as execution};
use astria_core::generated::astria::sequencerblock::v1::rollup_data::Value::{
    Deposit, SequencedData,
};
use astria_core::primitive::v1::RollupId;
use astria_core::Protobuf as _;
use bytes::Bytes;
use cnidarium::{StateDelta, Storage};
use prost::Message as _;
use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::info;

pub(crate) struct RollupExecutionService {
    pub storage: Storage,
}

#[async_trait::async_trait]
impl ExecutionService for RollupExecutionService {
    async fn get_genesis_info(
        self: Arc<Self>,
        request: Request<execution::GetGenesisInfoRequest>,
    ) -> Result<Response<execution::GenesisInfo>, Status> {
        println!("getting genesis info:");
        let _request = request.into_inner();
        let genesis_info = execution::GenesisInfo {
            rollup_id: Some(RollupId::new([69u8; 32]).into_raw()),
            sequencer_genesis_block_height: 2,
            celestia_block_variance: 100,
        };
        println!("{}", self.storage.latest_version());
        println!("genesis_info: {:?}", genesis_info);
        Ok(Response::new(genesis_info))
    }

    async fn get_block(
        self: Arc<Self>,
        request: Request<execution::GetBlockRequest>,
    ) -> Result<Response<execution::Block>, Status> {
        println!("getting block:");
        let request = request.into_inner();
        let snapshot = self.storage.latest_snapshot();
        let state_delta = StateDelta::new(snapshot);
        match request.identifier {
            Some(identidfier) => match identidfier.identifier {
                Some(id) => match id {
                    execution::block_identifier::Identifier::BlockNumber(height) => {
                        let block = state_delta.get_block(height).await.unwrap();
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
        _request: Request<execution::BatchGetBlocksRequest>,
    ) -> Result<Response<execution::BatchGetBlocksResponse>, Status> {
        // let request = request.into_inner();
        // let state = self.app.read().await;
        // let mut blocks = Vec::new();
        // for identifier in request.identifiers {
        //     match identifier.identifier {
        //         Some(id) => match id {
        //             execution::block_identifier::Identifier::BlockNumber(block_number) => {
        //                 blocks.push(state.get_block(block_number).unwrap().to_owned().into_raw());
        //             }
        //             execution::block_identifier::Identifier::BlockHash(_) => {
        //                 return Err(Status::unimplemented("Get Block by hash not implemented"))
        //             }
        //         },
        //         None => return Err(Status::invalid_argument("missing block identifier")),
        //     }
        // }
        Ok(Response::new(execution::BatchGetBlocksResponse::default()))
    }

    async fn execute_block(
        self: Arc<Self>,
        request: Request<execution::ExecuteBlockRequest>,
    ) -> Result<Response<execution::Block>, Status> {
        let request = request.into_inner();
        let timestamp = request.timestamp.unwrap();
        let mut transactions: Vec<Bytes> = Vec::new();
        for rollup_data in request.transactions {
            match rollup_data.value {
                Some(value) => match value {
                    SequencedData(data) => transactions.push(data),
                    Deposit(_) => {}
                },
                None => {}
            };
        }

        let snapshot = self.storage.latest_snapshot();
        let mut state_delta = StateDelta::new(snapshot);
        let commitment = state_delta.get_commitment_state().await.unwrap();
        let block_height = commitment.soft;
        info!("soft_height: {:?}", block_height);
        // Process transactions
        for tx in transactions {
            let raw_transaction =
                crate::generated::protocol::transaction::v1::Transaction::decode(tx.clone())
                    .unwrap();
            info!("decoded transaction: {:?}", raw_transaction);

            let transaction =
                crate::transaction::v1::Transaction::try_from_raw(raw_transaction).unwrap();
            let sender = transaction.verification_key().address_bytes();
            let actions = transaction.actions();
            for action in actions {
                match action {
                    crate::transaction::v1::Action::Transfer(transfer) => {
                        info!("executing transfer: {:?}", transfer);
                        execute_transfer(&transfer, sender, &mut state_delta)
                            .await
                            .unwrap();
                    }
                    crate::transaction::v1::Action::Text(send_text) => {
                        info!("executing send_text: {:?}", send_text);
                        execute_send_text(&send_text, sender, &mut state_delta)
                            .await
                            .unwrap();
                    }
                };
            }
        }
        // proccess_transactions(transactions, self.storage.clone())
        //     .await
        //     .unwrap();
        // execute new block
        let new_block = astria_core::generated::astria::execution::v1::Block {
            number: block_height + 1,
            parent_block_hash: Bytes::from_static(&[69u8; 32]),
            hash: Bytes::from_static(&[69u8; 32]),
            timestamp: Some(timestamp),
        };
        // save to state

        let block = Block::try_from_raw(new_block.clone()).unwrap();
        state_delta.put_block(block, block_height + 1).unwrap();
        let write_batch: cnidarium::StagedWriteBatch =
            self.storage.prepare_commit(state_delta).await.unwrap();
        let _hash = self.storage.commit_batch(write_batch).unwrap();
        Ok(Response::new(new_block))
    }

    async fn get_commitment_state(
        self: Arc<Self>,
        _request: Request<execution::GetCommitmentStateRequest>,
    ) -> Result<Response<execution::CommitmentState>, Status> {
        let snapshot = self.storage.latest_snapshot();
        let delta_state = StateDelta::new(snapshot);
        let commitment_state = delta_state.get_commitment_state().await.unwrap();
        info!("got commitment state: {:?}", commitment_state);
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
        // let mut state = self.app.write().await;
        let snapshot = self.storage.latest_snapshot();
        let mut state_delta = StateDelta::new(snapshot);
        let commitment_state_request = request.into_inner().commitment_state.unwrap();
        let soft_block_request = commitment_state_request.soft.as_ref().unwrap();
        let firm_block_request = commitment_state_request.firm.as_ref().unwrap();
        let soft_request = soft_block_request.number;
        let firm_request = firm_block_request.number;
        let soft_block = state_delta.get_block(soft_request).await.unwrap();
        let firm_block = state_delta.get_block(firm_request).await.unwrap();
        if soft_block.hash().to_owned() != soft_block_request.hash {
            println!(
                "soft block hash does not match: current: {:?},  request: {:?}",
                soft_block.hash().to_owned(),
                soft_block_request.hash
            );
            return Err(Status::invalid_argument("Soft block hash does not match"));
        }
        if firm_block.hash().to_owned() != firm_block_request.hash {
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

        // let games = self.game_manager.read().await;
        // let game_state = games.game_status(0);
        // println!("game 0 state: {:?}", game_state);
        Ok(Response::new(new_commitment_state))
    }
}
