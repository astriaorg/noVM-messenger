use astria_eyre::eyre::bail;
use borsh::{BorshDeserialize, BorshSerialize};

use super::{Value, ValueImpl};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(in crate::rollup) struct StorageVersion(u64);

impl From<u64> for StorageVersion {
    fn from(storage_version: u64) -> Self {
        StorageVersion(storage_version)
    }
}

impl From<StorageVersion> for u64 {
    fn from(storage_version: StorageVersion) -> Self {
        storage_version.0
    }
}

impl From<StorageVersion> for crate::storage::StoredValue<'_> {
    fn from(storage_version: StorageVersion) -> Self {
        crate::storage::StoredValue::Rollup(Value(ValueImpl::StorageVersion(storage_version)))
    }
}

impl TryFrom<crate::storage::StoredValue<'_>> for StorageVersion {
    type Error = astria_eyre::eyre::Error;

    fn try_from(value: crate::storage::StoredValue) -> Result<Self, Self::Error> {
        let crate::storage::StoredValue::Rollup(Value(ValueImpl::StorageVersion(storage_version))) =
            value
        else {
            bail!("app stored value type mismatch: expected storage version, found {value:?}");
        };
        Ok(storage_version)
    }
}
