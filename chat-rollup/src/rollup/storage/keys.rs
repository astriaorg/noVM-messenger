pub(in crate::rollup) const BLOCK: &str = "app/block";
pub(in crate::rollup) const BLOCK_HASH: &str = "app/block_hash";
pub(in crate::rollup) const BLOCK_HEIGHT: &str = "app/block_height";
pub(in crate::rollup) const BLOCK_TIMESTAMP: &str = "app/block_timestamp";
pub(in crate::rollup) const COMMITMENT_STATE: &str = "app/commitment_state";

pub(in crate::rollup) fn storage_version_by_height(height: u64) -> String {
    format!("app/storage_version/{height}")
}

pub(in crate::rollup) fn block(height: u32) -> String {
    format!("app/block/{height}")
}
