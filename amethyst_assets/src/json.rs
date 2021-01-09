use amethyst_error::{format_err, Error, ResultExt};
use serde::{Deserialize, Serialize};

use crate::Format;

/// Format for loading from JSON files. Mostly useful for prefabs.
/// This type can only be used as manually specified to the loader.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct JsonFormat;

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
