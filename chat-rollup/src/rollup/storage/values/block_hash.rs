use astria_eyre::eyre::bail;
use borsh::{BorshDeserialize, BorshSerialize};

use super::{Value, ValueImpl};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(in crate::rollup) struct BlockHash(String);

impl From<String> for BlockHash {
    fn from(block_hash: String) -> Self {
        BlockHash(block_hash)
    }
}

impl From<BlockHash> for String {
    fn from(block_hash: BlockHash) -> Self {
        block_hash.0
    }
}

impl<'a> From<BlockHash> for crate::storage::StoredValue<'a> {
    fn from(block_hash: BlockHash) -> Self {
        crate::storage::StoredValue::Rollup(Value(ValueImpl::BlockHash(block_hash)))
    }
}

impl<'a> TryFrom<crate::storage::StoredValue<'a>> for BlockHash {
    type Error = astria_eyre::eyre::Error;

    fn try_from(value: crate::storage::StoredValue) -> Result<Self, Self::Error> {
        let crate::storage::StoredValue::Rollup(Value(ValueImpl::BlockHash(block_hash))) = value
        else {
            bail!("app stored value type mismatch: expected block height, found {value:?}");
        };
        Ok(block_hash)
    }
}
