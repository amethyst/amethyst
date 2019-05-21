//! Dynamic Box<dyn Format<T>> serialization.
//!
//! This module implements serializers, deserializers and all the required
//! machinery to allow and loading asset formats from boxed trait types.
//! This is achieved by using `inventory` crate to store all registered pairs
//! of asset data types and their formats, and embedding the format name into
//! the serialization format itself.

use crate::Format;
use serde::{
    de::{self, DeserializeSeed, Expected, SeqAccess, Visitor},
    ser::SerializeTupleStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::BTreeMap, marker::PhantomData};

/// A trait for all asset types that have their format types.
/// Use this as a bound for asset data types when used inside boxed format types intended for deserialization.
/// registered with `register_format_type` macro.
///
///  This trait should never be implemented manually. Use the `register_format_type` macro to register it correctly.
/// ```ignore
/// // this must be used exactly once per data type
/// amethyst_assets::register_format_type!(AudioData);
///
/// // this must be used for every Format type impl that can be deserialized dynamically
/// amethyst_assets::register_format!("WAV", AudioData as WavFormat);
/// impl Format<AudioData> for WavFormat {
///     fn name(&self) -> &'static str {
///         "WAV"
///     }

///     fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
///         Ok(AudioData(bytes))
///     }
/// }
/// ```
pub trait FormatRegisteredData: 'static {
    // Used by deserialization. This is a private API.
    #[doc(hidden)]
    type Registration;
    #[doc(hidden)]
    fn get_registration(
        name: &'static str,
        deserializer: DeserializeFn<dyn Format<Self>>,
    ) -> Self::Registration;
    #[doc(hidden)]
    fn registry() -> &'static Registry<dyn Format<Self>>;
}

// Not public API. Used by macros.
#[doc(hidden)]
pub type DeserializeFn<T> =
    fn(&mut dyn erased_serde::Deserializer<'_>) -> erased_serde::Result<Box<T>>;

// Not public API. Used by macros.
#[doc(hidden)]
pub struct Registry<T: ?Sized> {
    pub map: BTreeMap<&'static str, Option<DeserializeFn<T>>>,
    pub names: Vec<&'static str>,
}

pub struct SeqLookupVisitor<'a, T: ?Sized + 'static> {
    pub expected: &'a dyn Expected,
    pub registry: &'static Registry<T>,
}

impl<'de, 'a, T: ?Sized + 'static> Visitor<'de> for SeqLookupVisitor<'a, T> {
    type Value = DeserializeFn<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Expected::fmt(self.expected, formatter)
    }

    fn visit_str<E: de::Error>(self, key: &str) -> Result<Self::Value, E> {
        match self.registry.map.get(key) {
            Some(Some(value)) => Ok(*value),
            Some(None) => Err(de::Error::custom(format_args!(
                "non-unique tag of {}: {:?}",
                self.expected, key
            ))),
            None => Err(de::Error::unknown_variant(key, &self.registry.names)),
        }
    }
}

impl<'de, 'a, T: ?Sized + 'static> DeserializeSeed<'de> for SeqLookupVisitor<'a, T> {
    type Value = DeserializeFn<T>;
    fn deserialize<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        de.deserialize_str(self)
    }
}

struct FnApply<T: ?Sized> {
    pub deserialize_fn: DeserializeFn<T>,
}

impl<'de, T: ?Sized> DeserializeSeed<'de> for FnApply<T> {
    type Value = Box<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut erased = erased_serde::Deserializer::erase(deserializer);
        (self.deserialize_fn)(&mut erased).map_err(de::Error::custom)
    }
}

struct FormatVisitor<D: FormatRegisteredData>(PhantomData<D>);

impl<'de, D: FormatRegisteredData> Visitor<'de> for FormatVisitor<D> {
    type Value = Box<dyn Format<D>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "dyn Format")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let seq_lookup = SeqLookupVisitor {
            expected: &self,
            registry: <D as FormatRegisteredData>::registry(),
        };
        let deserialize_fn = match seq.next_element_seed(seq_lookup)? {
            Some(deserialize_fn) => deserialize_fn,
            None => {
                return Err(de::Error::custom("expected tagged Format"));
            }
        };
        seq.next_element_seed(FnApply { deserialize_fn })?
            .ok_or_else(|| de::Error::invalid_length(1, &self))
    }
}

impl<D: FormatRegisteredData> Serialize for dyn Format<D> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut ser = serializer.serialize_tuple_struct("Format", 2)?;
        ser.serialize_field(self.name())?;
        ser.serialize_field(self)?;
        ser.end()
    }
}

impl<'de, D: FormatRegisteredData> Deserialize<'de> for Box<dyn Format<D>> {
    fn deserialize<DE: Deserializer<'de>>(
        deserializer: DE,
    ) -> std::result::Result<Self, DE::Error> {
        deserializer.deserialize_tuple_struct("Format", 2, FormatVisitor::<D>(PhantomData))
    }
}

impl<D: FormatRegisteredData> dyn Format<D> {
    // This code is called by `register_format` macro. Considered a private api otherwise.
    #[doc(hidden)]
    pub fn format_register(
        name: &'static str,
        deserializer: DeserializeFn<Self>,
    ) -> D::Registration {
        D::get_registration(name, deserializer)
    }
}

/// Register specific asset data types that can be deserialized with dynamic formats.
/// This is very useful for all assets that have any format types explicitly implemented.
/// Registered assets are used during loading of nested assets to determine format type
/// which will be used to deserialize that asset.
#[macro_export]
macro_rules! register_format_type {
    ($($asset_data:ty),*) => {
        $(
            const _register_format_type_impl: () = {
                $crate::inventory::collect!(AssetFormatRegistration);

                #[doc(hidden)]
                #[allow(unused)]
                pub struct AssetFormatRegistration {
                    name: &'static str,
                    deserializer: $crate::DeserializeFn<dyn $crate::Format<$asset_data>>,
                }

                impl $crate::FormatRegisteredData for $asset_data {
                    type Registration = AssetFormatRegistration;
                    fn get_registration(name: &'static str, deserializer: $crate::DeserializeFn<dyn $crate::Format<Self>>) -> Self::Registration {
                        AssetFormatRegistration { name, deserializer }
                    }
                    fn registry() -> &'static $crate::Registry<dyn $crate::Format<Self>> {
                        &REGISTRY
                    }
                }

                $crate::lazy_static::lazy_static! {
                    static ref REGISTRY: $crate::Registry<dyn $crate::Format<$asset_data>> = {
                        let mut map = std::collections::BTreeMap::new();
                        let mut names = std::vec::Vec::new();
                        for registered in $crate::inventory::iter::<AssetFormatRegistration> {
                            match map.entry(registered.name) {
                                std::collections::btree_map::Entry::Vacant(entry) => {
                                    entry.insert(std::option::Option::Some(registered.deserializer));
                                }
                                std::collections::btree_map::Entry::Occupied(mut entry) => {
                                    entry.insert(std::option::Option::None);
                                }
                            }
                            names.push(registered.name);
                        }
                        names.sort_unstable();
                        $crate::Registry { map, names }
                    };
                }
            };
        )*
    }
}

/// Register a dynamically deserializable format for given asset data type.
/// Note that provided asset data type must also be registered using `register_format_type` macro.
///
/// ```ignore
/// amethyst_assets::register_format!("WAV", WavFormat as AudioData);
/// ```
///
/// The provided name literal (`"WAV"` in example) must be identical to the one returned from `fn name` implementation.
/// This is required for right deserialization to be determined correctly. This parameter might be removed
/// in the future when some macro limitations will be worked around.
///
/// The `amethyst_assets` crate must be in scope in order to use that macro.
/// You can also specify name for the crate as additional first parameter.
///
/// ```ignore
/// amethyst_assets::register_format!(renamed_assets_crate; "WAV", WavFormat as AudioData);
/// ```
#[macro_export]
macro_rules! register_format {
    ($name:literal, $format:ty as $data:ty) => {
        $crate::register_format!(amethyst_assets; $name, $format as $data);
    };
    ($name:literal, $format:ty as $data:ty) => {
        $crate::register_format!(amethyst_assets; $name, $format as $data);
    };
    ($krate:ident; $name:literal, $format:ty as $data:ty) => {
        $crate::inventory::submit!{
            #![crate = $krate]
            <dyn $crate::Format<$data>>::format_register(
                $name,
                |deserializer| std::result::Result::Ok(
                    std::boxed::Box::new(
                        $crate::erased_serde::deserialize::<$format>(deserializer)?
                    ),
                ),
            )
        }
    };
}
