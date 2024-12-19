use super::{Value, ValueImpl};

#[allow(unused_imports)]
use astria_core::primitive::v1::asset::TracePrefixed as DomainTracePrefixed;
use astria_eyre::eyre::bail;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(crate) struct Text(String);

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Text(text.to_string())
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Text(text)
    }
}

impl From<Text> for String {
    fn from(text: Text) -> Self {
        text.0
    }
}

impl From<Text> for crate::storage::StoredValue<'_> {
    fn from(text: Text) -> Self {
        crate::storage::StoredValue::Text(Value(ValueImpl::Text(text)))
    }
}

impl TryFrom<crate::storage::StoredValue<'_>> for Text {
    type Error = astria_eyre::eyre::Error;

    fn try_from(value: crate::storage::StoredValue) -> Result<Self, Self::Error> {
        let crate::storage::StoredValue::Text(Value(ValueImpl::Text(text))) = value else {
            bail!("app stored value type mismatch: expected block height, found {value:?}");
        };
        Ok(text)
    }
}
