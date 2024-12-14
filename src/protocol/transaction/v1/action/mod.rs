use super::raw;
use crate::primitive::v1::{
    asset::{self},
    Address, AddressError,
};
use astria_core::{protocol::account, Protobuf};
pub mod group;

#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Deserialize, ::serde::Serialize),
    serde(into = "raw::Action", try_from = "raw::Action")
)]
pub enum Action {
    Transfer(Transfer),
    Text(SendText),
}

impl Protobuf for Action {
    type Error = Error;
    type Raw = raw::Action;

    #[must_use]
    fn to_raw(&self) -> Self::Raw {
        use raw::action::Value;
        let kind = match self {
            Action::Transfer(act) => Value::Transfer(act.to_raw()),
            Action::Text(act) => Value::Text(act.to_raw()),
        };
        raw::Action { value: Some(kind) }
    }

    /// Attempt to convert from a reference to raw, unchecked protobuf [`raw::Action`].
    ///
    /// # Errors
    ///
    /// Returns an error if conversion of one of the inner raw action variants
    /// to a native action fails.
    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Error> {
        Self::try_from_raw(raw.clone())
    }

    /// Attempt to convert from a raw, unchecked protobuf [`raw::Action`].
    ///
    /// # Errors
    ///
    /// Returns an error if conversion of one of the inner raw action variants
    /// to a native action fails.
    fn try_from_raw(proto: raw::Action) -> Result<Self, Error> {
        use raw::action::Value;
        let raw::Action { value } = proto;
        let Some(action) = value else {
            return Err(Error::unset());
        };
        let action = match action {
            Value::Transfer(act) => {
                Self::Transfer(Transfer::try_from_raw(act).map_err(Error::transfer)?)
            }
            Value::Text(account) => {
                Self::Text(SendText::try_from_raw(account).map_err(Error::send_text)?)
            }
        };
        Ok(action)
    }
}

// TODO: add unit tests for these methods (https://github.com/astriaorg/astria/issues/1593)
impl Action {
    #[must_use]
    pub fn as_transfer(&self) -> Option<&Transfer> {
        let Self::Transfer(transfer_action) = self else {
            return None;
        };
        Some(transfer_action)
    }
}

impl From<Transfer> for Action {
    fn from(value: Transfer) -> Self {
        Self::Transfer(value)
    }
}

impl From<SendText> for Action {
    fn from(value: SendText) -> Self {
        Self::Text(value)
    }
}

impl From<Action> for raw::Action {
    fn from(value: Action) -> Self {
        value.into_raw()
    }
}

impl TryFrom<raw::Action> for Action {
    type Error = Error;

    fn try_from(value: raw::Action) -> Result<Self, Self::Error> {
        Self::try_from_raw(value)
    }
}

// TODO: replace this trait with a Protobuf:FullName implementation.
// Issue tracked in #1567
pub(super) trait ActionName {
    fn name(&self) -> &'static str;
}

impl ActionName for Action {
    fn name(&self) -> &'static str {
        match self {
            Action::Transfer(_) => "Transfer",
            Action::Text(_) => "Text",
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(ActionErrorKind);

impl Error {
    fn unset() -> Self {
        Self(ActionErrorKind::Unset)
    }

    fn transfer(inner: TransferError) -> Self {
        Self(ActionErrorKind::Transfer(inner))
    }

    fn send_text(inner: SendTextError) -> Self {
        Self(ActionErrorKind::SendText(inner))
    }
}

#[derive(Debug, thiserror::Error)]
enum ActionErrorKind {
    #[error("required action value was not set")]
    Unset,
    #[error("transfer action was not valid")]
    Transfer(#[source] TransferError),
    #[error("send text action was not valid")]
    SendText(#[source] SendTextError),
}

#[derive(Clone, Debug)]
pub struct SendText {
    pub text: String,
    pub fee_asset: asset::Denom,
}

impl Protobuf for SendText {
    type Error = SendTextError;
    type Raw = raw::SendText;

    #[must_use]
    fn to_raw(&self) -> raw::SendText {
        let Self { text, fee_asset } = &self;
        raw::SendText {
            text: text.clone(),
            fee_asset: fee_asset.to_string(),
        }
    }

    /// Convert from a reference to the raw protobuf type.
    ///
    /// # Errors
    /// Returns `TextError` if the raw action's `to` address did not have the expected length.
    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Self::Error> {
        let raw::SendText { text, fee_asset } = raw;
        let text = text.to_string();
        let fee_asset = fee_asset.parse().map_err(SendTextError::fee_asset)?;
        Ok(Self { text, fee_asset })
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct SendTextError(SendTextActionErrorKind);

impl SendTextError {
    fn field_not_set(field: &'static str) -> Self {
        Self(SendTextActionErrorKind::FieldNotSet(field))
    }

    fn address(inner: AddressError) -> Self {
        Self(SendTextActionErrorKind::Address(inner))
    }

    fn asset(inner: asset::ParseDenomError) -> Self {
        Self(SendTextActionErrorKind::Asset(inner))
    }

    fn fee_asset(inner: asset::ParseDenomError) -> Self {
        Self(SendTextActionErrorKind::FeeAsset(inner))
    }
}

#[derive(Debug, thiserror::Error)]
enum SendTextActionErrorKind {
    #[error("the expected field in the raw source type was not set: `{0}`")]
    FieldNotSet(&'static str),
    #[error("`to` field did not contain a valid address")]
    Address(#[source] AddressError),
    #[error("`asset` field did not contain a valid asset ID")]
    Asset(#[source] asset::ParseDenomError),
    #[error("`fee_asset` field did not contain a valid asset ID")]
    FeeAsset(#[source] asset::ParseDenomError),
}

#[derive(Clone, Debug)]
pub struct Transfer {
    pub to: Address,
    pub amount: u128,
    /// asset to be transferred.
    pub asset: asset::Denom,
    /// asset to use for fee payment.
    pub fee_asset: asset::Denom,
}

impl Protobuf for Transfer {
    type Error = TransferError;
    type Raw = raw::Transfer;

    #[must_use]
    fn to_raw(&self) -> raw::Transfer {
        let Self {
            to,
            amount,
            asset,
            fee_asset,
        } = self;
        raw::Transfer {
            to: Some(to.to_raw()),
            amount: Some((*amount).into()),
            asset: asset.to_string(),
            fee_asset: fee_asset.to_string(),
        }
    }

    /// Convert from a reference to the raw protobuf type.
    ///
    /// # Errors
    /// Returns `TransferActionError` if the raw action's `to` address did not have the expected
    /// length.
    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Self::Error> {
        let raw::Transfer {
            to,
            amount,
            asset,
            fee_asset,
        } = raw;
        let Some(to) = to else {
            return Err(TransferError::field_not_set("to"));
        };
        let to = Address::try_from_raw(to).map_err(TransferError::address)?;
        let amount = amount.map_or(0, Into::into);
        let asset = asset.parse().map_err(TransferError::asset)?;
        let fee_asset = fee_asset.parse().map_err(TransferError::fee_asset)?;

        Ok(Self {
            to,
            amount,
            asset,
            fee_asset,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct TransferError(TransferActionErrorKind);

impl TransferError {
    fn field_not_set(field: &'static str) -> Self {
        Self(TransferActionErrorKind::FieldNotSet(field))
    }

    fn address(inner: AddressError) -> Self {
        Self(TransferActionErrorKind::Address(inner))
    }

    fn asset(inner: asset::ParseDenomError) -> Self {
        Self(TransferActionErrorKind::Asset(inner))
    }

    fn fee_asset(inner: asset::ParseDenomError) -> Self {
        Self(TransferActionErrorKind::FeeAsset(inner))
    }
}

#[derive(Debug, thiserror::Error)]
enum TransferActionErrorKind {
    #[error("the expected field in the raw source type was not set: `{0}`")]
    FieldNotSet(&'static str),
    #[error("`to` field did not contain a valid address")]
    Address(#[source] AddressError),
    #[error("`asset` field did not contain a valid asset ID")]
    Asset(#[source] asset::ParseDenomError),
    #[error("`fee_asset` field did not contain a valid asset ID")]
    FeeAsset(#[source] asset::ParseDenomError),
}
