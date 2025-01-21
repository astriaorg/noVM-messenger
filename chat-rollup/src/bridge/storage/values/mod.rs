pub(in crate::bridge) mod address_bytes;
pub(super) use address_bytes::AddressBytes;
use borsh::{BorshDeserialize, BorshSerialize};
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(crate) struct Value<'a>(ValueImpl<'a>);

#[derive(Debug, BorshSerialize, BorshDeserialize)]
enum ValueImpl<'a> {
    AddressBytes(AddressBytes<'a>),
}
