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
pub mod primitive {
    pub mod v1 {
        include!("astria.primitive.v1.rs");

        #[cfg(feature = "serde")]
        mod _serde_impl {
            use super::*;
            include!("astria.primitive.v1.serde.rs");
        }
    }
}

#[path = ""]
pub mod protocol {
    #[path = ""]
    pub mod transaction {
        pub mod v1 {
            include!("astria.protocol.transaction.v1.rs");

            #[cfg(feature = "serde")]
            mod _serde_impl {
                use super::*;
                include!("astria.protocol.transaction.v1.serde.rs");
            }
        }
    }
}
