mod block;
mod block_hash;
mod block_height;
mod block_timestamp;
mod commitment_state;
mod storage_version;
use borsh::{BorshDeserialize, BorshSerialize};

pub(in crate::rollup) use self::block::Block;
pub(in crate::rollup) use self::block_hash::BlockHash;
pub(in crate::rollup) use self::block_height::BlockHeight;
pub(in crate::rollup) use self::block_timestamp::BlockTimestamp;
pub(in crate::rollup) use self::commitment_state::CommitmentState;
pub(in crate::rollup) use self::commitment_state::CommitmentStateHeight;
pub(in crate::rollup) use self::storage_version::StorageVersion;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(crate) struct Value(ValueImpl);

#[derive(Debug, BorshSerialize, BorshDeserialize)]
enum ValueImpl {
    Block(Block),
    BlockHeight(BlockHeight),
    BlockHash(BlockHash),
    CommitmentState(CommitmentState),
    CommitmentStateHeight(CommitmentStateHeight),
    BlockTimestamp(BlockTimestamp),
    StorageVersion(StorageVersion),
}
