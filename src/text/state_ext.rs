use std::borrow::Cow;

use crate::primitive::v1::asset;
use astria_eyre::{
    anyhow_to_eyre,
    eyre::{bail, Result, WrapErr as _},
};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use tracing::instrument;

use self::{
    storage::values,
    values::{LastText, Text},
};

use super::storage::{
    self,
    keys::{self},
};
use crate::storage::StoredValue;

#[async_trait]
pub(crate) trait StateReadExt: StateRead {
    #[instrument(skip_all)]
    async fn get_text(&self, id: u64) -> Result<Text> {
        let Some(bytes) = self
            .get_raw(&keys::text(id))
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to read raw text from state")?
        else {
            bail!("text not found state");
        };
        StoredValue::deserialize(&bytes)
            .and_then(|value| storage::values::Text::try_from(value).map(Text::from))
            .context("invalid block height bytes")
    }
    #[instrument(skip_all)]
    async fn get_last_text_id(&self) -> Result<LastText> {
        let Some(bytes) = self
            .get_raw(keys::LAST_TX)
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to read last text id from state")?
        else {
            bail!("last text id not found state");
        };
        StoredValue::deserialize(&bytes)
            .and_then(|value| storage::values::LastText::try_from(value).map(LastText::from))
            .context("invalid block height bytes")
    }
}

impl<T: ?Sized + StateRead> StateReadExt for T {}

#[async_trait]
pub(crate) trait StateWriteExt: StateWrite {
    #[instrument(skip_all)]
    fn put_text(&mut self, text: String, id: u64) -> Result<()> {
        let bytes = StoredValue::from(storage::values::Text::from(text.as_str()))
            .serialize()
            .context("failed to serialize text")?;
        self.put_raw(keys::text(id).to_string(), bytes);
        Ok(())
    }

    #[instrument(skip_all)]
    fn put_last_text_id(&mut self, id: u64) -> Result<()> {
        let bytes = StoredValue::from(storage::values::LastText::from(id))
            .serialize()
            .context("failed to serialize last text id")?;
        self.put_raw(keys::LAST_TX.to_string(), bytes);
        Ok(())
    }
}

impl<T: StateWrite> StateWriteExt for T {}
