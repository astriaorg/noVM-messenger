use super::{Value, ValueImpl};
use astria_eyre::eyre::bail;
use borsh::io::{Read, Write};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CommitmentStateHeight {
    pub soft: u32,
    pub firm: u32,
    pub celestia: u32,
}

#[derive(Debug)]
pub struct CommitmentState(CommitmentStateHeight);

impl From<CommitmentStateHeight> for CommitmentState {
    fn from(commitment: CommitmentStateHeight) -> Self {
        CommitmentState(commitment)
    }
}

impl From<CommitmentState> for CommitmentStateHeight {
    fn from(commit: CommitmentState) -> Self {
        commit.0
    }
}

impl BorshSerialize for CommitmentState {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.serialize(writer)
    }
}

impl BorshDeserialize for CommitmentState {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let commitment = CommitmentStateHeight::deserialize_reader(reader)?;
        Ok(CommitmentState::from(commitment))
    }
}

impl From<CommitmentState> for crate::storage::StoredValue<'_> {
    fn from(commitment: CommitmentState) -> Self {
        crate::storage::StoredValue::Rollup(Value(ValueImpl::CommitmentState(commitment)))
    }
}

impl TryFrom<crate::storage::StoredValue<'_>> for CommitmentState {
    type Error = astria_eyre::eyre::Error;

    fn try_from(value: crate::storage::StoredValue) -> Result<Self, Self::Error> {
        let crate::storage::StoredValue::Rollup(Value(ValueImpl::CommitmentState(commitment))) =
            value
        else {
            bail!("app stored value type mismatch: expected block height, found {value:?}");
        };
        Ok(commitment)
    }
}
