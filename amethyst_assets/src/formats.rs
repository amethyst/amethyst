use serde::Deserialize;

use error::{Error, ResultExt};
use {Asset, SimpleFormat};

/// Format for loading from Ron files.
#[derive(Default, Clone, Debug)]
pub struct RonFormat;

impl<T> SimpleFormat<T> for RonFormat
where
    T: Asset,
    T::Data: for<'a> Deserialize<'a> + Send + Sync + 'static,
{
    const NAME: &'static str = "Ron";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<T::Data, Error> {
        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&bytes).chain_err(|| "Failed deserializing Ron file")?;
        let val = T::Data::deserialize(&mut d).chain_err(|| "Failed parsing Ron file")?;
        d.end().chain_err(|| "Failed parsing Ron file")?;

        Ok(val)
    }
}
