use std::io::Read;

use distill::importer::{
    self as importer, BoxedImporter, ImportOp, ImportedAsset, Importer, ImporterValue, SerdeObj,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::{AssetUuid, Format};

/// A simple state for Importer to retain the same UUID between imports
/// for all single-asset source files
#[derive(Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "6b00ea4b-f98c-4b43-94e1-e696c96a6b93"]
pub struct SimpleImporterState {
    id: Option<AssetUuid>,
}

/// Wrapper struct to be able to impl Importer for any SimpleFormat
pub struct SimpleImporter<A: 'static, T: Format<A> + TypeUuid>(
    pub T,
    ::std::marker::PhantomData<A>,
);

impl<A: 'static, T: Format<A> + TypeUuid + 'static> From<T> for SimpleImporter<A, T> {
    fn from(fmt: T) -> SimpleImporter<A, T> {
        SimpleImporter(fmt, ::std::marker::PhantomData)
    }
}
impl<A, T: Format<A> + TypeUuid + Send + 'static> TypeUuid for SimpleImporter<A, T>
where
    A: SerdeObj,
{
    const UUID: type_uuid::Bytes = T::UUID;
}

impl<A, T: Format<A> + TypeUuid + Send + 'static> Importer for SimpleImporter<A, T>
where
    A: SerdeObj,
{
    type State = SimpleImporterState;
    type Options = T;

    fn version_static() -> u32
    where
        Self: Sized,
    {
        1
    }
    fn version(&self) -> u32 {
        Self::version_static()
    }

    fn import(
        &self,
        _op: &mut ImportOp,
        source: &mut dyn Read,
        options: &Self::Options,
        state: &mut Self::State,
    ) -> importer::Result<ImporterValue> {
        if state.id.is_none() {
            state.id = Some(AssetUuid(*uuid::Uuid::new_v4().as_bytes()));
        }
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;
        let import_result = options
            .import_simple(bytes)
            .map_err(|e| importer::Error::Boxed(e.into_error()))?;
        Ok(ImporterValue {
            assets: vec![ImportedAsset {
                id: state.id.expect("AssetUUID not generated"),
                search_tags: Vec::new(),
                build_deps: Vec::new(),
                load_deps: Vec::new(),
                asset_data: Box::new(import_result),
                build_pipeline: None,
            }],
        })
    }
}
/// Use [inventory::submit!] to register an importer to use for a file extension.
#[derive(Debug)]
pub struct SourceFileImporter {
    /// File extension for this type of file
    pub extension: &'static str,
    /// closure that creates Importer for given Format
    pub instantiator: fn() -> Box<dyn BoxedImporter>,
}
inventory::collect!(SourceFileImporter);

/// Get the registered importers and their associated extension.
pub fn get_source_importers(
) -> impl Iterator<Item = (&'static str, Box<dyn BoxedImporter + 'static>)> {
    inventory::iter::<SourceFileImporter>
        .into_iter()
        .map(|s| (s.extension.trim_start_matches('.'), (s.instantiator)()))
}

/// Associates the given file extension with a `Format` implementation
///
/// The `AssetDaemon` will automatically re-import the asset when a file of that format is created
/// or modified.
///
/// # Parameters
///
/// * `ext`: File extension including the leading `.`, such as `".ron"`.
/// * `format`: Type that implements the `Format` trait.
///
/// # Examples
///
/// ```
/// use amethyst::assets::Format;
/// use amethyst::error::Error;
/// use serde::{Deserialize, Serialize};
/// use type_uuid::TypeUuid;
///
/// #[derive(Clone, Debug, Serialize, Deserialize, TypeUuid)]
/// #[uuid = "00000000-0000-0000-0000-000000000000"]
/// pub struct AudioData(pub Vec<u8>);
///
/// #[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, TypeUuid)]
/// #[uuid = "00000000-0000-0000-0000-000000000000"]
/// pub struct WavFormat;
/// impl Format<AudioData> for WavFormat {
///     fn name(&self) -> &'static str {
///         "WAV"
///     }

///     fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
///         Ok(AudioData(bytes))
///     }
/// }
/// amethyst_assets::register_importer!(".wav", WavFormat);
/// ```
#[macro_export]
macro_rules! register_importer {
    ($ext:literal, $format:ty) => {
        $crate::register_importer!(amethyst_assets; $ext, $format);
    };
    ($krate:ident; $ext:literal, $format:ty) => {
        $crate::inventory::submit!{
            #![crate = $krate]
            $crate::SourceFileImporter {
                extension: $ext,
                instantiator: || Box::new($crate::SimpleImporter::from(<$format as Default>::default())),
            }
        }
    };
}
