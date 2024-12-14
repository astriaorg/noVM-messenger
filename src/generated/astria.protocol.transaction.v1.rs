#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Action {
    #[prost(oneof = "action::Value", tags = "1, 2")]
    pub value: ::core::option::Option<action::Value>,
}
/// Nested message and enum types in `Action`.
pub mod action {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// Core protocol actions are defined on 1-10
        #[prost(message, tag = "1")]
        Transfer(super::Transfer),
        #[prost(message, tag = "2")]
        Text(super::SendText),
    }
}
impl ::prost::Name for Action {
    const NAME: &'static str = "Action";
    const PACKAGE: &'static str = "astria.protocol.transaction.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("astria.protocol.transaction.v1.{}", Self::NAME)
    }
}
/// `TransferAction` represents a value transfer transaction.
///
/// Note: all values must be set (ie. not `None`), otherwise it will
/// be considered invalid by the rolup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transfer {
    #[prost(message, optional, tag = "1")]
    pub to: ::core::option::Option<super::super::super::primitive::v1::Address>,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<super::super::super::primitive::v1::Uint128>,
    /// the asset to be transferred
    #[prost(string, tag = "3")]
    pub asset: ::prost::alloc::string::String,
    /// the asset used to pay the transaction fee
    #[prost(string, tag = "4")]
    pub fee_asset: ::prost::alloc::string::String,
}
impl ::prost::Name for Transfer {
    const NAME: &'static str = "Transfer";
    const PACKAGE: &'static str = "astria.protocol.transaction.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("astria.protocol.transaction.v1.{}", Self::NAME)
    }
}
/// 'SendTextAction' represents a bet transaction.
///
/// Note: all values must be set (ie. not `None`), otherwise it will
/// be considered invalid by the rollup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendText {
    #[prost(string, tag = "1")]
    pub text: ::prost::alloc::string::String,
    /// the asset used to pay the transaction fee
    #[prost(string, tag = "2")]
    pub fee_asset: ::prost::alloc::string::String,
}
impl ::prost::Name for SendText {
    const NAME: &'static str = "SendText";
    const PACKAGE: &'static str = "astria.protocol.transaction.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("astria.protocol.transaction.v1.{}", Self::NAME)
    }
}
/// `Transaction` is a transaction `TransactionBody` together with a public
/// ket and a signature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(bytes = "bytes", tag = "1")]
    pub signature: ::prost::bytes::Bytes,
    #[prost(bytes = "bytes", tag = "2")]
    pub public_key: ::prost::bytes::Bytes,
    #[prost(message, optional, tag = "3")]
    pub body: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for Transaction {
    const NAME: &'static str = "Transaction";
    const PACKAGE: &'static str = "astria.protocol.transaction.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("astria.protocol.transaction.v1.{}", Self::NAME)
    }
}
/// The `TransactionBody` of the `Transaction` that is being signed over.
/// It contains transaction `TransactionParams` and `Actions`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionBody {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<TransactionParams>,
    #[prost(message, repeated, tag = "2")]
    pub actions: ::prost::alloc::vec::Vec<Action>,
}
impl ::prost::Name for TransactionBody {
    const NAME: &'static str = "TransactionBody";
    const PACKAGE: &'static str = "astria.protocol.transaction.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("astria.protocol.transaction.v1.{}", Self::NAME)
    }
}
/// The `TransactionParams` of the transaction that define the
/// validity of the `Transaction`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionParams {
    #[prost(uint32, tag = "1")]
    pub nonce: u32,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for TransactionParams {
    const NAME: &'static str = "TransactionParams";
    const PACKAGE: &'static str = "astria.protocol.transaction.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("astria.protocol.transaction.v1.{}", Self::NAME)
    }
}
