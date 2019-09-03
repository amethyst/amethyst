use crate::Format;
use amethyst_error::{format_err, Error, ResultExt};
use serde::{Deserialize, Serialize};

/// Format for loading from RON files. Mostly useful for prefabs.
/// This type cannot be used for tagged deserialization.
/// It is meant to be used at top-level loading, manually specified to the loader.
/// ```rust,ignore
/// loader.load("prefab.ron", RonFormat, ());
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RonFormat;

impl<D> Format<D> for RonFormat
where
    D: for<'a> Deserialize<'a> + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        "Ron"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<D, Error> {
        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&bytes)
            .with_context(|_| format_err!("Failed deserializing Ron file"))?;
        let val =
            D::deserialize(&mut d).with_context(|_| format_err!("Failed parsing Ron file"))?;
        d.end()
            .with_context(|_| format_err!("Failed parsing Ron file"))?;

        Ok(val)
    }
}

/// Format for loading from JSON files. Mostly useful for prefabs.
/// This type can only be used as manually specified to the loader.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct JsonFormat;

#[cfg(feature = "json")]
impl<D> Format<D> for JsonFormat
where
    D: for<'a> Deserialize<'a> + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        "Json"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<D, Error> {
        use serde_json::de::Deserializer;
        let mut d = Deserializer::from_slice(&bytes);
        let val = D::deserialize(&mut d)
            .with_context(|_| format_err!("Failed deserializing Json file"))?;
        d.end()
            .with_context(|_| format_err!("Failed deserializing Json file"))?;

        Ok(val)
    }
}
