use astria_core::execution::v1::Block;
use astria_eyre::{
    anyhow_to_eyre,
    eyre::{bail, Result, WrapErr as _},
};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use tracing::{info, instrument};

use self::storage::CommitmentStateHeight;

use super::storage::{self, keys};
use crate::storage::StoredValue;

#[async_trait]
pub(crate) trait StateReadExt: StateRead {
    #[instrument(skip_all)]
    async fn get_block_height(&self) -> Result<u64> {
        let Some(bytes) = self
            .get_raw(keys::BLOCK_HEIGHT)
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to read raw block_height from state")?
        else {
            bail!("block height not found state");
        };
        StoredValue::deserialize(&bytes)
            .and_then(|value| storage::BlockHeight::try_from(value).map(u64::from))
            .context("invalid block height bytes")
    }

    #[instrument(skip_all)]
    async fn get_block_hash(&self) -> Result<String> {
        let Some(bytes) = self
            .get_raw(keys::BLOCK_HASH)
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to read raw block_height from state")?
        else {
            bail!("block height not found state");
        };
        StoredValue::deserialize(&bytes)
            .map(|value| storage::BlockHash::try_from(value).map(String::from))
            .context("invalid block height bytes")
            .unwrap()
    }

    #[instrument(skip_all)]
    async fn get_block(&self, height: u32) -> Result<Block> {
        let Some(bytes) = self
            .get_raw(&keys::block(height))
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to read raw block_height from state")?
        else {
            bail!("block height not found state");
        };
        StoredValue::deserialize(&bytes)
            .and_then(|value| storage::Block::try_from(value).map(Block::from))
            .context("invalid block height bytes")
    }

    async fn get_commitment_state(&self) -> Result<CommitmentStateHeight> {
        let Some(bytes) = self
            .get_raw(keys::COMMITMENT_STATE)
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to read raw commitment state from state")?
        else {
            bail!("commitment state not found state");
        };
        StoredValue::deserialize(&bytes)
            .and_then(storage::CommitmentState::try_from)
            .map(CommitmentStateHeight::from)
            .context("invalid commitment state bytes")
    }

    async fn get_storage_version_by_height(&self, height: u64) -> Result<u64> {
        let Some(bytes) = self
            .get_raw(&keys::storage_version_by_height(height))
            .await
            .map_err(anyhow_to_eyre)
            .wrap_err("failed to read raw storage version from state")?
        else {
            bail!("storage version not found state");
        };
        StoredValue::deserialize(&bytes)
            .and_then(|value| storage::StorageVersion::try_from(value).map(u64::from))
            .context("invalid storage version bytes")
    }
}

impl<T: StateRead> StateReadExt for T {}

#[async_trait]
pub(crate) trait StateWriteExt: StateWrite {
    #[instrument(skip_all)]
    fn put_block_height(&mut self, height: u64) -> Result<()> {
        let bytes = StoredValue::from(storage::BlockHeight::from(height))
            .serialize()
            .context("failed to serialize block height")?;
        self.put_raw(keys::BLOCK_HEIGHT.to_string(), bytes);
        Ok(())
    }

    #[instrument(skip_all)]
    fn put_commitment_state(&mut self, soft: u32, firm: u32, celestia: u32) -> Result<()> {
        let bytes = StoredValue::from(storage::CommitmentState::from(CommitmentStateHeight {
            soft,
            firm,
            celestia,
        }))
        .serialize()
        .context("failed to serialize commitment state")?;
        info!("bytes: {:?}", bytes);
        self.put_raw(keys::COMMITMENT_STATE.to_string(), bytes);
        Ok(())
    }

    #[instrument(skip_all)]
    fn put_block(&mut self, block: Block, height: u32) -> Result<()> {
        let bytes = StoredValue::from(storage::Block::from(block))
            .serialize()
            .context("failed to serialize block")?;
        self.put_raw(keys::block(height), bytes);
        Ok(())
    }
}

impl<T: StateWrite> StateWriteExt for T {}

#[cfg(test)]
mod tests {
    use cnidarium::StateDelta;

    use super::*;

    #[tokio::test]
    async fn block_height() {
        let storage = cnidarium::TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = StateDelta::new(snapshot);

        // doesn't exist at first
        let _ = state
            .get_block_height()
            .await
            .expect_err("no block height should exist at first");

        // can write new
        let block_height_orig = 0;
        state.put_block_height(block_height_orig).unwrap();
        assert_eq!(
            state
                .get_block_height()
                .await
                .expect("a block height was written and must exist inside the database"),
            block_height_orig,
            "stored block height was not what was expected"
        );

        // can rewrite with new value
        let block_height_update = 1;
        state.put_block_height(block_height_update).unwrap();
        assert_eq!(
            state
                .get_block_height()
                .await
                .expect("a new block height was written and must exist inside the database"),
            block_height_update,
            "updated block height was not what was expected"
        );
    }

    #[tokio::test]
    async fn put_and_get_commitment_state() {
        let storage = cnidarium::TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = StateDelta::new(snapshot);

        state.put_commitment_state(0, 0, 2).unwrap();
        let commit = state.get_commitment_state().await.unwrap();
        assert_eq!(0, commit.soft);
        assert_eq!(0, commit.firm);
        assert_eq!(2, commit.celestia);
    }
}
