use amethyst_core::specs::error::Error as SpecsError;
use amethyst_core::specs::{Entity, WriteStorage};
use amethyst_core::{GlobalTransform, Transform, TransformPrefabData};

use super::PrefabData;

macro_rules! impl_data {
    ( $($ty:ident:$i:tt),* ) => {
        impl<'a, $($ty),*> PrefabData<'a> for ( $( Option<$ty> , )* )
            where $( $ty : PrefabData<'a> ),*
        {
            type SystemData = (
                $(
                    $ty::SystemData,
                )*
            );

            fn load_prefab(
                &self,
                entity: Entity,
                system_data: &mut Self::SystemData,
                entities: &[Entity],
            ) -> Result<(), SpecsError> {
                #![allow(unused_variables)]
                $(
                    if let Some(ref prefab) = self.$i {
                        prefab.load_prefab(entity, &mut system_data.$i, entities)?;
                    }
                )*
                Ok(())
            }
        }

        impl<'a, $($ty),*> PrefabData<'a> for ( $( $ty , )* )
            where $( $ty : PrefabData<'a> ),*
        {
            type SystemData = (
                $(
                    $ty::SystemData,
                )*
            );

            fn load_prefab(
                &self,
                entity: Entity,
                system_data: &mut Self::SystemData,
                entities: &[Entity],
            ) -> Result<(), SpecsError> {
                #![allow(unused_variables)]
                $(
                    self.$i.load_prefab(entity, &mut system_data.$i, entities)?;
                )*
                Ok(())
            }
        }
    };
}

impl<'a> PrefabData<'a> for GlobalTransform {
    type SystemData = WriteStorage<'a, Self>;

    fn load_prefab(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), SpecsError> {
        storage.insert(entity, self.clone()).map(|_| ())
    }
}

impl<'a> PrefabData<'a> for Transform {
    type SystemData = WriteStorage<'a, Self>;

    fn load_prefab(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), SpecsError> {
        storage.insert(entity, self.clone()).map(|_| ())
    }
}

impl<'a> PrefabData<'a> for TransformPrefabData {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, GlobalTransform>,
    );

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), SpecsError> {
        system_data.0.insert(entity, self.transform.clone())?;
        system_data.1.insert(entity, GlobalTransform::default())?;
        Ok(())
    }
}

impl_data!(A:0);
impl_data!(A:0, B:1);
impl_data!(A:0, B:1, C:2);
impl_data!(A:0, B:1, C:2, D:3);
impl_data!(A:0, B:1, C:2, D:3, E:4);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19);
impl_data!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19, U:20);
