//! This module contains a few serialization shims for types from `palette` crate.
//! It is required due to buggy default implementation of those types.
//! See this issue for more details: https://github.com/Ogeon/palette/issues/130
//! When above issue will be resolved, this can probably be removed.
#![allow(clippy::many_single_char_names)]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Srgb serialization shim.
/// ```
/// # use serde::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize)]
/// struct MyType(#[serde(with = "amethyst_rendy::serde_shim::srgb")] pub palette::Srgb);
/// ```
pub mod srgb {
    use super::*;
    #[derive(Serialize, Deserialize)]
    struct Srgb(f32, f32, f32);

    /// Serialize Srgba type as tuple struct with three floats
    pub fn serialize<S: Serializer>(x: &palette::Srgb, s: S) -> Result<S::Ok, S::Error> {
        let (r, g, b) = x.into_components();
        Srgb(r, g, b).serialize(s)
    }

    /// Deserialize Srgba type as tuple struct with three floats
    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<palette::Srgb, D::Error> {
        let t = Srgb::deserialize(de)?;
        Ok(palette::Srgb::new(t.0, t.1, t.2))
    }
}

/// Srgba serialization shim.
/// ```
/// # use serde::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize)]
/// struct MyType(#[serde(with = "amethyst_rendy::serde_shim::srgba")] pub palette::Srgba);
/// ```
pub mod srgba {
    use super::*;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
    struct Srgba(f32, f32, f32, f32);

    /// Serialize Srgba type as tuple struct with four floats
    pub fn serialize<S: Serializer>(x: &palette::Srgba, s: S) -> Result<S::Ok, S::Error> {
        let (r, g, b, a) = x.into_components();
        Srgba(r, g, b, a).serialize(s)
    }

    /// Deserialize Srgba type as tuple struct with four floats
    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<palette::Srgba, D::Error> {
        let t = Srgba::deserialize(de)?;
        Ok(palette::Srgba::new(t.0, t.1, t.2, t.3))
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct MySrgbaWrapper(#[serde(with = "crate::serde_shim::srgba")] palette::Srgba);

    #[derive(Serialize, Deserialize)]
    struct MySrgbWrapper(#[serde(with = "crate::serde_shim::srgb")] palette::Srgb);

    #[test]
    fn deserialize_srgba() {
        let de: MySrgbaWrapper =
            ron::de::from_str("MySrgbaWrapper(Srgba(0.1, 0.2, 0.3, 0.4))").unwrap();
        assert_eq!(de.0, palette::Srgba::new(0.1, 0.2, 0.3, 0.4))
    }
    #[test]
    fn deserialize_srgb() {
        let de: MySrgbWrapper = ron::de::from_str("MySrgbWrapper(Srgb(0.1, 0.2, 0.3))").unwrap();
        assert_eq!(de.0, palette::Srgb::new(0.1, 0.2, 0.3))
    }
    #[test]
    fn serialize_srgb() {
        let val = MySrgbWrapper(palette::Srgb::new(0.1, 0.2, 0.3));
        assert_eq!(
            ron::ser::to_string_pretty(&val, Default::default()).unwrap(),
            "((0.1, 0.2, 0.3))"
        )
    }
    #[test]
    fn serialize_srgba() {
        let val = MySrgbaWrapper(palette::Srgba::new(0.1, 0.2, 0.3, 0.4));
        assert_eq!(
            ron::ser::to_string_pretty(&val, Default::default()).unwrap(),
            "((0.1, 0.2, 0.3, 0.4))"
        )
    }
}
