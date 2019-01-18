use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, ResultExt},
    Asset, SimpleFormat,
};

/// Format for loading from Ron files.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
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
        let mut d =
            Deserializer::from_bytes(&bytes).chain_err(|| "Failed deserializing Ron file")?;
        let val = T::Data::deserialize(&mut d).chain_err(|| "Failed parsing Ron file")?;
        d.end().chain_err(|| "Failed parsing Ron file")?;

        Ok(val)
    }
}

/// Format for loading from Json files.
#[cfg(feature = "json")]
#[derive(Default, Clone, Debug)]
pub struct JsonFormat;

#[cfg(feature = "json")]
impl<T> SimpleFormat<T> for JsonFormat
where
    T: Asset,
    T::Data: for<'a> Deserialize<'a> + Send + Sync + 'static,
{
    const NAME: &'static str = "Json";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<T::Data, Error> {
        use serde_json::de::Deserializer;
        let mut d = Deserializer::from_slice(&bytes);
        let val = T::Data::deserialize(&mut d).chain_err(|| "Failed deserializing Json file")?;
        d.end().chain_err(|| "Failed parsing Json file")?;

        Ok(val)
    }
}
