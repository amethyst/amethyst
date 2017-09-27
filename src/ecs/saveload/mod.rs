//! Save and load entites from various formats with serde

mod de;
mod ser;
mod details;

use self::details::{Components, EntityData, SerializableComponent, Storages};

pub mod marker;

pub use self::de::{deserialize, WorldDeserialize};
pub use self::details::{NoError, SaveLoadComponent};
pub use self::ser::{serialize, serialize_recursive, WorldSerialize};
