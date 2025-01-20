use crate::generated::protocol::genesis::v1 as raw;
use astria_core::primitive::v1::Address;
use astria_core::primitive::v1::AddressError;
use astria_core::Protobuf;
/// The genesis state of Astria's Sequencer.
///
/// Verified to only contain valid fields (right now, addresses that have the same base prefix
/// as set in `GenesisState::address_prefixes::base`).
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(try_from = "raw::GenesisAppState", into = "raw::GenesisAppState")
)]
pub struct GenesisAppState {
    rollup_name: String,
    sequencer_genesis_block_height: u32,
    celestia_genesis_block_height: u32,
    celestia_block_variance: u64,
    accounts: Vec<Account>,
    authority_sudo_address: astria_core::primitive::v1::Address,
}

impl GenesisAppState {
    #[must_use]
    pub fn accounts(&self) -> &[Account] {
        &self.accounts
    }

    #[must_use]
    pub fn authority_sudo_address(&self) -> &Address {
        &self.authority_sudo_address
    }

    #[must_use]
    pub fn rollup_name(&self) -> &str {
        &self.rollup_name
    }

    #[must_use]
    pub fn sequencer_genesis_block_height(&self) -> u32 {
        self.sequencer_genesis_block_height
    }

    #[must_use]
    pub fn celestia_genesis_block_height(&self) -> u32 {
        self.celestia_genesis_block_height
    }

    #[must_use]
    pub fn celestia_block_variance(&self) -> u64 {
        self.celestia_block_variance
    }
}

impl Protobuf for GenesisAppState {
    type Error = GenesisAppStateError;
    type Raw = raw::GenesisAppState;

    // TODO (https://github.com/astriaorg/astria/issues/1580): remove this once Rust is upgraded to/past 1.83
    #[expect(
        clippy::allow_attributes,
        clippy::allow_attributes_without_reason,
        reason = "false positive on `allowed_fee_assets` due to \"allow\" in the name"
    )]
    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Self::Error> {
        let Self::Raw {
            accounts,
            authority_sudo_address,
            rollup_name,
            sequencer_genesis_block_height,
            celestia_genesis_block_height,
            celestia_block_variance,
        } = raw;
        let accounts = accounts
            .iter()
            .map(Account::try_from_raw_ref)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Self::Error::accounts)?;

        let authority_sudo_address = authority_sudo_address
            .as_ref()
            .ok_or_else(|| Self::Error::field_not_set("authority_sudo_address"))
            .and_then(|addr| {
                Address::try_from_raw_ref(addr).map_err(Self::Error::authority_sudo_address)
            })?;

        let this = Self {
            accounts,
            authority_sudo_address,
            rollup_name: rollup_name.clone(),
            sequencer_genesis_block_height: *sequencer_genesis_block_height,
            celestia_genesis_block_height: *celestia_genesis_block_height,
            celestia_block_variance: *celestia_block_variance,
        };
        Ok(this)
    }

    fn to_raw(&self) -> Self::Raw {
        let Self {
            accounts,
            authority_sudo_address,
            rollup_name,
            sequencer_genesis_block_height,
            celestia_genesis_block_height,
            celestia_block_variance,
        } = self;
        Self::Raw {
            accounts: accounts.iter().map(Account::to_raw).collect(),
            authority_sudo_address: Some(authority_sudo_address.to_raw()),
            rollup_name: rollup_name.clone(),
            sequencer_genesis_block_height: *sequencer_genesis_block_height,
            celestia_genesis_block_height: *celestia_genesis_block_height,

            celestia_block_variance: *celestia_block_variance,
        }
    }
}

impl TryFrom<raw::GenesisAppState> for GenesisAppState {
    type Error = <Self as Protobuf>::Error;

    fn try_from(value: raw::GenesisAppState) -> Result<Self, Self::Error> {
        Self::try_from_raw(value)
    }
}

impl From<GenesisAppState> for raw::GenesisAppState {
    fn from(value: GenesisAppState) -> Self {
        value.into_raw()
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct GenesisAppStateError(GenesisAppStateErrorKind);

impl GenesisAppStateError {
    fn accounts(source: AccountError) -> Self {
        Self(GenesisAppStateErrorKind::Accounts { source })
    }

    fn authority_sudo_address(source: AddressError) -> Self {
        Self(GenesisAppStateErrorKind::AuthoritySudoAddress { source })
    }

    fn field_not_set(name: &'static str) -> Self {
        Self(GenesisAppStateErrorKind::FieldNotSet { name })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed ensuring invariants of {}", GenesisAppState::full_name())]
enum GenesisAppStateErrorKind {
    #[error("`accounts` field was invalid")]
    Accounts { source: AccountError },
    #[error("`authority_sudo_address` field was invalid")]
    AuthoritySudoAddress { source: AddressError },
    #[error("field was not set: `{name}`")]
    FieldNotSet { name: &'static str },
}

#[derive(Clone, Copy, Debug)]
pub struct Account {
    pub address: Address,
    pub balance: u128,
}

impl Protobuf for Account {
    type Error = AccountError;
    type Raw = raw::Account;

    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Self::Error> {
        let Self::Raw { address, balance } = raw;
        let address = address
            .as_ref()
            .ok_or_else(|| AccountError::field_not_set("address"))
            .and_then(|addr| Address::try_from_raw_ref(addr).map_err(Self::Error::address))?;
        let balance = balance
            .ok_or_else(|| AccountError::field_not_set("balance"))
            .map(Into::into)?;
        Ok(Self { address, balance })
    }

    fn to_raw(&self) -> Self::Raw {
        let Self { address, balance } = self;
        Self::Raw {
            address: Some(address.to_raw()),
            balance: Some((*balance).into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct AccountError(AccountErrorKind);

impl AccountError {
    fn address(source: AddressError) -> Self {
        Self(AccountErrorKind::Address { source })
    }

    fn field_not_set(name: &'static str) -> Self {
        Self(AccountErrorKind::FieldNotSet { name })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed ensuring invariants of {}", Account::full_name())]
enum AccountErrorKind {
    #[error("`address` field was invalid")]
    Address { source: AddressError },
    #[error("field was not set: `{name}`")]
    FieldNotSet { name: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;
    use astria_core::primitive::v1::Address;

    const ASTRIA_ADDRESS_PREFIX: &str = "astria";

    fn alice() -> Address {
        Address::builder()
            .prefix(ASTRIA_ADDRESS_PREFIX)
            .slice(hex::decode("1c0c490f1b5528d8173c5de46d131160e4b2c0c3").unwrap())
            .try_build()
            .unwrap()
    }

    fn bob() -> Address {
        Address::builder()
            .prefix(ASTRIA_ADDRESS_PREFIX)
            .slice(hex::decode("34fec43c7fcab9aef3b3cf8aba855e41ee69ca3a").unwrap())
            .try_build()
            .unwrap()
    }

    fn charlie() -> Address {
        Address::builder()
            .prefix(ASTRIA_ADDRESS_PREFIX)
            .slice(hex::decode("60709e2d391864b732b4f0f51e387abb76743871").unwrap())
            .try_build()
            .unwrap()
    }

    fn proto_genesis_state() -> raw::GenesisAppState {
        raw::GenesisAppState {
            accounts: vec![
                raw::Account {
                    address: Some(alice().to_raw()),
                    balance: Some(1_000_000_000_000_000_000.into()),
                },
                raw::Account {
                    address: Some(bob().to_raw()),
                    balance: Some(1_000_000_000_000_000_000.into()),
                },
                raw::Account {
                    address: Some(charlie().to_raw()),
                    balance: Some(1_000_000_000_000_000_000.into()),
                },
            ],
            authority_sudo_address: Some(alice().to_raw()),
            rollup_name: "astria-1".to_string(),
            sequencer_genesis_block_height: 0,
            celestia_genesis_block_height: 0,
            celestia_block_variance: 0,
        }
    }

    fn genesis_state() -> GenesisAppState {
        proto_genesis_state().try_into().unwrap()
    }

    #[cfg(feature = "serde")]
    #[test]
    fn genesis_state_is_unchanged() {
        insta::assert_json_snapshot!("genesis_state", genesis_state());
    }
}
