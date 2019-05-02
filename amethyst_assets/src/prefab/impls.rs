use amethyst_core::{
    ecs::{Entity, WriteStorage},
    math::RealField,
    Named, Transform,
};
use amethyst_error::Error;

use crate::{PrefabData, ProgressCounter};

impl<'a, T> PrefabData<'a> for Option<T>
where
    T: PrefabData<'a>,
{
    type SystemData = <T as PrefabData<'a>>::SystemData;
    type Result = Option<<T as PrefabData<'a>>::Result>;

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<Self::Result, Error> {
        if let Some(ref prefab) = self {
            Ok(Some(prefab.add_to_entity(
                entity,
                system_data,
                entities,
                children,
            )?))
        } else {
            Ok(None)
        }
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        if let Some(ref mut prefab) = self {
            prefab.load_sub_assets(progress, system_data)
        } else {
            Ok(false)
        }
    }
}

impl<'a, N: RealField> PrefabData<'a> for Transform<N> {
    type SystemData = WriteStorage<'a, Transform<N>>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storages: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storages.insert(entity, self.clone()).map(|_| ())?;
        Ok(())
    }
}

impl<'a> PrefabData<'a> for Named {
    type SystemData = (WriteStorage<'a, Named>,);
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storages: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storages.0.insert(entity, self.clone()).map(|_| ())?;
        Ok(())
    }
}

macro_rules! impl_data {
    ( $($ty:ident:$i:tt),* ) => {
        #[allow(unused)]
        impl<'a, $($ty),*> PrefabData<'a> for ( $( $ty , )* )
            where $( $ty : PrefabData<'a> ),*
        {
            type SystemData = (
                $(
                    $ty::SystemData,
                )*
            );
            type Result = ();

            fn add_to_entity(
                &self,
                entity: Entity,
                system_data: &mut Self::SystemData,
                entities: &[Entity],
                children: &[Entity],
            ) -> Result<(), Error> {
                #![allow(unused_variables)]
                $(
                    self.$i.add_to_entity(entity, &mut system_data.$i, entities, children)?;
                )*
                Ok(())
            }

            fn load_sub_assets(
                &mut self, progress:
                &mut ProgressCounter,
                system_data: &mut Self::SystemData
            ) -> Result<bool, Error> {
                let mut ret = false;
                $(
                    if self.$i.load_sub_assets(progress, &mut system_data.$i)? {
                        ret = true;
                    }
                )*
                Ok(ret)
            }
        }
    };
}

impl_data!();
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
