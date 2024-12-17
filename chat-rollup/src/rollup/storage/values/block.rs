use super::{BlockTimestamp, Value, ValueImpl};
use astria_core::execution::v1::Block as ExeBlock;
use astria_core::generated::astria::execution::v1::Block as RawBlock;

use astria_core::Protobuf;
use astria_eyre::eyre::bail;
use borsh::io::{Read, Write};
use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;
#[derive(Debug)]
pub(in crate::rollup) struct Block(ExeBlock);

impl From<ExeBlock> for Block {
    fn from(block: ExeBlock) -> Self {
        Block(block)
    }
}

impl From<Block> for ExeBlock {
    fn from(block: Block) -> Self {
        block.0
    }
}

impl BorshSerialize for Block {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let block = self.0.clone();
        let raw_block = block.into_raw();
        let timestamp = raw_block.timestamp.unwrap();
        let block_timestamp = BlockTimestamp::from(timestamp);
        BorshSerialize::serialize(&raw_block.number, writer).unwrap();
        BorshSerialize::serialize(&raw_block.hash.to_vec(), writer).unwrap();
        BorshSerialize::serialize(&raw_block.parent_block_hash.to_vec(), writer).unwrap();
        BorshSerialize::serialize(&block_timestamp, writer)
    }
}

impl BorshDeserialize for Block {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let number = u32::deserialize_reader(reader)?;
        let hash = Vec::<u8>::deserialize_reader(reader)?;
        let parent_block_hash = Vec::<u8>::deserialize_reader(reader)?;
        let timestamp = BlockTimestamp::deserialize_reader(reader)?;
        let raw_block = RawBlock {
            number,
            hash: Bytes::from(hash),
            parent_block_hash: Bytes::from(parent_block_hash),
            timestamp: Some(timestamp.into()),
        };
        let exe_block = ExeBlock::try_from_raw(raw_block).unwrap();
        Ok(Block::from(exe_block))
    }
}

impl<'a> From<Block> for crate::storage::StoredValue<'a> {
    fn from(block: Block) -> Self {
        crate::storage::StoredValue::Rollup(Value(ValueImpl::Block(block)))
    }
}

impl<'a> TryFrom<crate::storage::StoredValue<'a>> for Block {
    type Error = astria_eyre::eyre::Error;

    fn try_from(value: crate::storage::StoredValue) -> Result<Self, Self::Error> {
        let crate::storage::StoredValue::Rollup(Value(ValueImpl::Block(block))) = value else {
            bail!("app stored value type mismatch: expected block height, found {value:?}");
        };
        Ok(block)
    }
}
