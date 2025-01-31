use crate::{accounts::AddressBytes, storage::keys::AccountPrefixer};

pub(in crate::bridge) const BRIDGE_ACCOUNT_PREFIX: &str = "bridge/account";

/// Example: `accounts/gGhH....zZ4=/balance/`.
///                   |base64 chars|
#[allow(dead_code)]
pub(in crate::bridge) fn bridge<TAddress: AddressBytes>(address: &TAddress) -> String {
    format!("{}", AccountPrefixer::new(BRIDGE_ACCOUNT_PREFIX, address))
}
