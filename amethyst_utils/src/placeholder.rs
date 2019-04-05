use std::marker::PhantomData;
use amethyst_rendy::rendy::texture::image::ImageTextureConfig;
use amethyst_rendy::types::Texture;

#[derive(Deserialize, Serialize)]
pub struct GraphicsPrefab<B, V, M = ObjFormat, T = ImageFormat>
{p1: PhantomData<B>, p2: PhantomData<V>, p3: PhantomData<M>, p4: PhantomData<T>}

impl<'a, B, V, M, T> PrefabData<'a> for GraphicsPrefab<B, V, M, T>
    where
        B: Backend,
        M: Format<Mesh<B>>,
        M::Options: DeserializeOwned + Serialize,
        T: Format<Texture<B>, Options = ImageTextureConfig>,
{
    type SystemData = ();
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut <Self as PrefabData<'_>>::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        unimplemented!()
    }
}
impl<B, V, M, T> Default for GraphicsPrefab<B, V, M, T>
{
    fn default() -> Self {
        GraphicsPrefab{p1: PhantomData, p2: PhantomData, p3: PhantomData, p4: PhantomData}
    }
}
