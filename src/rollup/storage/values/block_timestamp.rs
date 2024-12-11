use astria_eyre::eyre::bail;
use borsh::{
    io::{Read, Write},
    BorshDeserialize, BorshSerialize,
};

use super::{Value, ValueImpl};

#[derive(Debug)]
pub(in crate::rollup) struct BlockTimestamp(pbjson_types::Timestamp);

impl From<pbjson_types::Timestamp> for BlockTimestamp {
    fn from(block_timestamp: pbjson_types::Timestamp) -> Self {
        BlockTimestamp(block_timestamp)
    }
}

impl From<BlockTimestamp> for pbjson_types::Timestamp {
    fn from(block_timestamp: BlockTimestamp) -> Self {
        block_timestamp.0
    }
}

impl BorshSerialize for BlockTimestamp {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.0.seconds, writer).unwrap();
        BorshSerialize::serialize(&self.0.nanos, writer).unwrap();
        Ok(())
    }
}

impl BorshDeserialize for BlockTimestamp {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let seconds = i64::deserialize_reader(reader)?;
        let nanos = i32::deserialize_reader(reader)?;
        let timestamp = pbjson_types::Timestamp {
            seconds: seconds,
            nanos: nanos,
        };
        Ok(BlockTimestamp(timestamp))
    }
}

impl From<BlockTimestamp> for crate::storage::StoredValue<'_> {
    fn from(block_timestamp: BlockTimestamp) -> Self {
        crate::storage::StoredValue::Rollup(Value(ValueImpl::BlockTimestamp(block_timestamp)))
    }
}

impl<'a> TryFrom<crate::storage::StoredValue<'a>> for BlockTimestamp {
    type Error = astria_eyre::eyre::Error;

    fn try_from(value: crate::storage::StoredValue<'a>) -> Result<Self, Self::Error> {
        let crate::storage::StoredValue::Rollup(Value(ValueImpl::BlockTimestamp(block_timestamp))) =
            value
        else {
            bail!("app stored value type mismatch: expected block timestamp, found {value:?}");
        };
        Ok(block_timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_round_trip() {
        let timestamp = BlockTimestamp(pbjson_types::Timestamp {
            seconds: 1_000_000_000,
            nanos: 56,
        });
        let serialized = borsh::to_vec(&timestamp).unwrap();
        eprintln!("{:?}", serialized);
        let deserialized: BlockTimestamp = borsh::from_slice(&serialized).unwrap();
        eprintln!("{:?}", deserialized);
        assert_eq!(timestamp.0, deserialized.0);
    }
}
