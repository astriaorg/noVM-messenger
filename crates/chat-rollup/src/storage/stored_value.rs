use astria_eyre::{eyre::WrapErr as _, Result};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(crate) enum StoredValue<'a> {
    Unit,
    Rollup(crate::rollup::storage::Value),
    Account(crate::accounts::storage::Value),
    Asset(crate::assets::storage::Value<'a>),
    Address(crate::address::storage::Value<'a>),
    Text(crate::text::storage::Value),
    Bridge(crate::bridge::storage::Value<'a>),
}

impl StoredValue<'_> {
    pub(crate) fn serialize(&self) -> Result<Vec<u8>> {
        borsh::to_vec(&self).wrap_err("failed to serialize stored value")
    }

    pub(crate) fn deserialize(bytes: &[u8]) -> Result<Self> {
        borsh::from_slice(bytes).wrap_err("failed to deserialize stored value")
    }
}
