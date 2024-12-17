use base64_serde::base64_serde_type;
use serde::Serializer;
pub mod test_utils;

use prost::Name;

#[cfg(not(target_pointer_width = "64"))]
compile_error!(
    "library is only guaranteed to run on 64 bit machines due to casts from/to u64 and usize"
);

pub(crate) mod serde;

/// A trait to convert from raw decoded protobuf types to idiomatic astria types.
///
/// The primary use of this trait is to convert to/from foreign types.
pub trait Protobuf: Sized {
    /// Errors that can occur when transforming from a raw type.
    type Error;
    /// The raw deserialized protobuf type.
    type Raw: prost::Name;

    /// Convert from a reference to the raw protobuf type.
    ///
    /// # Errors
    /// Returns [`Self::Error`] as defined by the implementor of this trait.
    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Self::Error>;

    /// Convert from the raw protobuf type, dropping it.
    ///
    /// This method provides a default implementation in terms of
    /// [`Self::try_from_raw_ref`].
    ///
    /// # Errors
    /// Returns [`Self::Error`] as defined by the implementor of this trait.
    fn try_from_raw(raw: Self::Raw) -> Result<Self, Self::Error> {
        Self::try_from_raw_ref(&raw)
    }

    /// Convert to the raw protobuf type by reference.
    fn to_raw(&self) -> Self::Raw;

    /// Convert to the raw protobuf type, dropping `self`.
    ///
    /// This method provides a default implementation in terms of
    /// [`Self::to_raw`].
    fn into_raw(self) -> Self::Raw {
        Self::to_raw(&self)
    }

    #[must_use]
    fn full_name() -> String {
        Self::Raw::full_name()
    }
}

base64_serde_type!(pub(crate) Base64Standard, base64::engine::general_purpose::STANDARD);
pub(crate) fn base64_serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    Base64Standard::serialize(value, serializer)
}
