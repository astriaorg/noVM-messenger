use astria_eyre::eyre::bail;
use borsh::{BorshDeserialize, BorshSerialize};

use super::{Value, ValueImpl};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(in crate::rollup) struct BlockHeight(u64);

impl From<u64> for BlockHeight {
    fn from(block_height: u64) -> Self {
        BlockHeight(block_height)
    }
}

impl From<BlockHeight> for u64 {
    fn from(block_height: BlockHeight) -> Self {
        block_height.0
    }
}

impl From<BlockHeight> for crate::storage::StoredValue<'_> {
    fn from(block_height: BlockHeight) -> Self {
        crate::storage::StoredValue::Rollup(Value(ValueImpl::BlockHeight(block_height)))
    }
}

impl TryFrom<crate::storage::StoredValue<'_>> for BlockHeight {
    type Error = astria_eyre::eyre::Error;

    fn try_from(value: crate::storage::StoredValue) -> Result<Self, Self::Error> {
        let crate::storage::StoredValue::Rollup(Value(ValueImpl::BlockHeight(block_height))) =
            value
        else {
            bail!("app stored value type mismatch: expected block height, found {value:?}");
        };
        Ok(block_height)
    }
}
