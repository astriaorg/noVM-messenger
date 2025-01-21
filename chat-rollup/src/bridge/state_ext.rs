use std::{
    borrow::Cow,
    fmt::Display,
    pin::Pin,
    task::{ready, Context, Poll},
};

use crate::{
    accounts::AddressBytes,
    storage::{self, StoredValue},
};
use astria_core::{
    crypto::ADDRESS_LENGTH,
    primitive::v1::{
        asset::{self, IbcPrefixed, TracePrefixed},
        Address,
    },
};
use astria_eyre::{
    anyhow_to_eyre,
    eyre::{OptionExt as _, Result, WrapErr as _},
};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use futures::Stream;
use pin_project_lite::pin_project;
use tracing::{debug, instrument};

use super::storage::keys;

pub(crate) fn nria() -> TracePrefixed {
    "nria".parse().unwrap()
}

#[allow(dead_code)]
pub(crate) fn astria_address(bytes: &[u8]) -> Address {
    let address: Address = Address::builder()
        .prefix("astria")
        .slice(bytes)
        .try_build()
        .unwrap();
    address
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct AssetBalance {
    pub(crate) asset: asset::IbcPrefixed,
    pub(crate) balance: u128,
}

pin_project! {
    /// A stream of IBC prefixed assets and their balances for a given account.
    pub(crate) struct AccountAssetBalancesStream<St> {
        #[pin]
        underlying: St,
    }
}

#[async_trait]
pub(crate) trait StateReadExt: StateRead + crate::assets::StateReadExt {
    #[instrument(skip_all, fields(address = %address.display_address()))]
    async fn is_bridge<T: AddressBytes>(&self, address: &T) -> Result<bool> {
        self.get_bridge_account(address)
            .await
            .map(|account| account.is_some())
    }

    #[instrument(skip_all, fields(bridge_address = %bridge_address.display_address()))]
    async fn get_bridge_account<T: AddressBytes>(
        &self,
        bridge_address: &T,
    ) -> Result<Option<[u8; ADDRESS_LENGTH]>> {
        let Some(bytes) = self
            .get_raw(&keys::bridge(bridge_address))
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed reading raw bridge account sudo address from state")?
        else {
            debug!("bridge account sudo address not found, returning None");
            return Ok(None);
        };
        StoredValue::deserialize(&bytes)
            .and_then(|value| {
                crate::bridge::storage::values::address_bytes::AddressBytes::try_from(value).map(
                    |stored_address_bytes| Some(<[u8; ADDRESS_LENGTH]>::from(stored_address_bytes)),
                )
            })
            .wrap_err("invalid bridge account sudo address bytes")
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[async_trait]
pub(crate) trait StateWriteExt: StateWrite {
    #[instrument(skip_all)]
    fn put_bridge_account<T>(&mut self, bridge_address: &T) -> Result<()>
    where
        T: AddressBytes,
    {
        let bytes = StoredValue::from(
            crate::bridge::storage::values::address_bytes::AddressBytes::from(
                bridge_address.address_bytes(),
            ),
        )
        .serialize()
        .context("failed to serialize bridge account sudo address")?;
        self.put_raw(keys::bridge(bridge_address), bytes);
        Ok(())
    }
}

impl<T: StateWrite> StateWriteExt for T {}
