use crate::generated::protocol as raw;
use borsh::{BorshDeserialize, BorshSerialize};
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Deserialize, ::serde::Serialize),
    serde(into = "raw::Action", try_from = "raw::Action")
)]
pub enum Action {
    Transfer(raw::transaction::v1::Transfer),
    Up(Up),
    Down(Down),
}
#[derive(Debug, Clone)]
pub struct Up {
    pub amount: u128,
    pub pool_id: u64,
}
#[derive(Debug, Clone)]
pub struct Down {
    pub amount: u128,
    pub pool_id: u64,
}
