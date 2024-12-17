#![allow(
    unreachable_pub,
    clippy::pedantic,
    clippy::needless_borrows_for_generic_args,
    clippy::arithmetic_side_effects
)]
//! Files generated using [`tonic-build`] and [`buf`] via the [`tools/protobuf-compiler`]
//! build tool.
//!
//! [`tonic-build`]: https://docs.rs/tonic-build
//! [`buf`]: https://buf.build
//! [`tools/protobuf-compiler`]: ../../../../tools/protobuf-compiler

#[path = ""]
pub mod protocol {
    #[path = ""]
    pub mod transaction {
        pub mod v1 {
            include!("astria.protocol.transaction.v1.rs");

            mod _serde_impl {
                use super::*;
                include!("astria.protocol.transaction.v1.serde.rs");
            }
        }
    }
}
