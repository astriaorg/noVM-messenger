use super::{Value, ValueImpl};
use astria_eyre::eyre::bail;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(crate) struct LastText(u64);

impl From<u64> for LastText {
    fn from(id: u64) -> Self {
        LastText(id)
    }
}

impl From<LastText> for u64 {
    fn from(id: LastText) -> Self {
        id.0
    }
}

impl<'a> From<LastText> for crate::storage::StoredValue<'a> {
    fn from(id: LastText) -> Self {
        crate::storage::StoredValue::Text(Value(ValueImpl::LastText(id)))
    }
}

impl<'a> TryFrom<crate::storage::StoredValue<'a>> for LastText {
    type Error = astria_eyre::eyre::Error;

    fn try_from(value: crate::storage::StoredValue) -> Result<Self, Self::Error> {
        let crate::storage::StoredValue::Text(Value(ValueImpl::LastText(text))) = value else {
            bail!("app stored value type mismatch: expected block height, found {value:?}");
        };
        Ok(text)
    }
}
