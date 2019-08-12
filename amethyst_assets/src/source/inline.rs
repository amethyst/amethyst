
use std::collections::HashMap;
use amethyst_error::{format_err, Error, ResultExt};
use crate::{error, source::Source};

/// Directory source.
///
/// Please note that there is a default directory source
/// inside the `Loader`, which is automatically used when you call
/// `load`. In case you want another, second, directory for assets,
/// you can instantiate one yourself, too. Please use `Loader::load_from` then.
#[derive(Debug)]
pub struct Inline {
    assets: HashMap<String, Vec<u8>>,
}

impl Inline {
    /// Creates a new inline storage.
    pub fn new() -> Self {
        let assets = vec![
            // ("font/square.ttf".to_string(), include_bytes!("../../../examples/assets/font/square.ttf").to_vec()),
            // ("img/asset_loading.png".to_string(), include_bytes!("../../../examples/assets/img/asset_loading.png").to_vec()),
            // ("img/gltf.png".to_string(), include_bytes!("../../../examples/assets/img/gltf.png").to_vec()),
            // ("img/material.png".to_string(), include_bytes!("../../../examples/assets/img/material.png").to_vec()),
            // ("img/pong.png".to_string(), include_bytes!("../../../examples/assets/img/pong.png").to_vec()),
            // ("img/renderable.png".to_string(), include_bytes!("../../../examples/assets/img/renderable.png").to_vec()),
            // ("img/sphere.png".to_string(), include_bytes!("../../../examples/assets/img/sphere.png").to_vec()),
            // ("img/ui.png".to_string(), include_bytes!("../../../examples/assets/img/ui.png").to_vec()),
            // ("img/window.png".to_string(), include_bytes!("../../../examples/assets/img/window.png").to_vec()),
            // ("large/AlphaBlendModeTest.glb".to_string(), include_bytes!("../../../examples/assets/large/AlphaBlendModeTest.glb").to_vec()),
            // ("large/crystal_stone/scene.bin".to_string(), include_bytes!("../../../examples/assets/large/crystal_stone/scene.bin").to_vec()),
            // ("large/crystal_stone/scene.gltf".to_string(), include_bytes!("../../../examples/assets/large/crystal_stone/scene.gltf").to_vec()),
            // ("large/crystal_stone/textures/Stone_low_1_baseColor.png".to_string(), include_bytes!("../../../examples/assets/large/crystal_stone/textures/Stone_low_1_baseColor.png").to_vec()),
            // ("large/crystal_stone/textures/Stone_low_1_emissive.png".to_string(), include_bytes!("../../../examples/assets/large/crystal_stone/textures/Stone_low_1_emissive.png").to_vec()),
            // ("large/crystal_stone/textures/Stone_low_1_metallicRoughness.png".to_string(), include_bytes!("../../../examples/assets/large/crystal_stone/textures/Stone_low_1_metallicRoughness.png").to_vec()),
            // ("large/crystal_stone/textures/Stone_low_1_normal.png".to_string(), include_bytes!("../../../examples/assets/large/crystal_stone/textures/Stone_low_1_normal.png").to_vec()),
            // ("large/InterpolationTest.glb".to_string(), include_bytes!("../../../examples/assets/large/InterpolationTest.glb").to_vec()),
            // ("large/NormalTangentMirrorTest.glb".to_string(), include_bytes!("../../../examples/assets/large/NormalTangentMirrorTest.glb").to_vec()),
            // ("large/NormalTangentTest.glb".to_string(), include_bytes!("../../../examples/assets/large/NormalTangentTest.glb").to_vec()),
            // ("large/SciFiHelmet/SciFiHelmet.bin".to_string(), include_bytes!("../../../examples/assets/large/SciFiHelmet/SciFiHelmet.bin").to_vec()),
            // ("large/SciFiHelmet/SciFiHelmet.gltf".to_string(), include_bytes!("../../../examples/assets/large/SciFiHelmet/SciFiHelmet.gltf").to_vec()),
            // ("large/SciFiHelmet/SciFiHelmet_AmbientOcclusion.png".to_string(), include_bytes!("../../../examples/assets/large/SciFiHelmet/SciFiHelmet_AmbientOcclusion.png").to_vec()),
            // ("large/SciFiHelmet/SciFiHelmet_BaseColor.png".to_string(), include_bytes!("../../../examples/assets/large/SciFiHelmet/SciFiHelmet_BaseColor.png").to_vec()),
            // ("large/SciFiHelmet/SciFiHelmet_MetallicRoughness.png".to_string(), include_bytes!("../../../examples/assets/large/SciFiHelmet/SciFiHelmet_MetallicRoughness.png").to_vec()),
            // ("large/SciFiHelmet/SciFiHelmet_Normal.png".to_string(), include_bytes!("../../../examples/assets/large/SciFiHelmet/SciFiHelmet_Normal.png").to_vec()),
            // ("LICENSE_ASSETS.md".to_string(), include_bytes!("../../../examples/assets/LICENSE_ASSETS.md").to_vec()),
            // ("locale/locale_en.ftl".to_string(), include_bytes!("../../../examples/assets/locale/locale_en.ftl").to_vec()),
            // ("locale/locale_fr.ftl".to_string(), include_bytes!("../../../examples/assets/locale/locale_fr.ftl").to_vec()),
            // ("mesh/Box.gltf".to_string(), include_bytes!("../../../examples/assets/mesh/Box.gltf").to_vec()),
            // ("mesh/Box0.bin".to_string(), include_bytes!("../../../examples/assets/mesh/Box0.bin").to_vec()),
            // ("mesh/cone.obj".to_string(), include_bytes!("../../../examples/assets/mesh/cone.obj").to_vec()),
            // ("mesh/cube.obj".to_string(), include_bytes!("../../../examples/assets/mesh/cube.obj").to_vec()),
            // ("mesh/cuboid.custom".to_string(), include_bytes!("../../../examples/assets/mesh/cuboid.custom").to_vec()),
            // ("mesh/ground.dds".to_string(), include_bytes!("../../../examples/assets/mesh/ground.dds").to_vec()),
            // ("mesh/lid.obj".to_string(), include_bytes!("../../../examples/assets/mesh/lid.obj").to_vec()),
            // ("mesh/Monster.gltf".to_string(), include_bytes!("../../../examples/assets/mesh/Monster.gltf").to_vec()),
            // ("mesh/Monster.png".to_string(), include_bytes!("../../../examples/assets/mesh/Monster.png").to_vec()),
            // ("mesh/Monster0.bin".to_string(), include_bytes!("../../../examples/assets/mesh/Monster0.bin").to_vec()),
            // ("mesh/puffy.bin".to_string(), include_bytes!("../../../examples/assets/mesh/puffy.bin").to_vec()),
            // ("mesh/puffy.gltf".to_string(), include_bytes!("../../../examples/assets/mesh/puffy.gltf").to_vec()),
            // ("mesh/rectangle.obj".to_string(), include_bytes!("../../../examples/assets/mesh/rectangle.obj").to_vec()),
            // ("mesh/sphere.obj".to_string(), include_bytes!("../../../examples/assets/mesh/sphere.obj").to_vec()),
            // ("mesh/teapot.obj".to_string(), include_bytes!("../../../examples/assets/mesh/teapot.obj").to_vec()),
            // ("prefab/animation.ron".to_string(), include_bytes!("../../../examples/assets/prefab/animation.ron").to_vec()),
            // ("prefab/arc_ball_camera.ron".to_string(), include_bytes!("../../../examples/assets/prefab/arc_ball_camera.ron").to_vec()),
            // ("prefab/auto_fov.ron".to_string(), include_bytes!("../../../examples/assets/prefab/auto_fov.ron").to_vec()),
            // ("prefab/example.ron".to_string(), include_bytes!("../../../examples/assets/prefab/example.ron").to_vec()),
            // ("prefab/fly_camera.ron".to_string(), include_bytes!("../../../examples/assets/prefab/fly_camera.ron").to_vec()),
            // ("prefab/monster_scene.ron".to_string(), include_bytes!("../../../examples/assets/prefab/monster_scene.ron").to_vec()),
            // ("prefab/prefab_adapter.ron".to_string(), include_bytes!("../../../examples/assets/prefab/prefab_adapter.ron").to_vec()),
            // ("prefab/prefab_basic.ron".to_string(), include_bytes!("../../../examples/assets/prefab/prefab_basic.ron").to_vec()),
            // ("prefab/prefab_custom.ron".to_string(), include_bytes!("../../../examples/assets/prefab/prefab_custom.ron").to_vec()),
            // ("prefab/prefab_multi.ron".to_string(), include_bytes!("../../../examples/assets/prefab/prefab_multi.ron").to_vec()),
            // ("prefab/puffy_scene.ron".to_string(), include_bytes!("../../../examples/assets/prefab/puffy_scene.ron").to_vec()),
            // ("prefab/renderable.ron".to_string(), include_bytes!("../../../examples/assets/prefab/renderable.ron").to_vec()),
            // ("prefab/rendy_example_scene.ron".to_string(), include_bytes!("../../../examples/assets/prefab/rendy_example_scene.ron").to_vec()),
            // ("prefab/sphere.ron".to_string(), include_bytes!("../../../examples/assets/prefab/sphere.ron").to_vec()),
            // ("prefab/spotlights_scene.ron".to_string(), include_bytes!("../../../examples/assets/prefab/spotlights_scene.ron").to_vec()),
            // ("prefab/sprite_animation.ron".to_string(), include_bytes!("../../../examples/assets/prefab/sprite_animation.ron").to_vec()),
            // ("prefab/sprite_animation_test.ron".to_string(), include_bytes!("../../../examples/assets/prefab/sprite_animation_test.ron").to_vec()),
            // ("texture/arrow_semi_transparent.png".to_string(), include_bytes!("../../../examples/assets/texture/arrow_semi_transparent.png").to_vec()),
            // ("texture/bat.32x32.png".to_string(), include_bytes!("../../../examples/assets/texture/bat.32x32.png").to_vec()),
            // ("texture/bat_semi_transparent.png".to_string(), include_bytes!("../../../examples/assets/texture/bat_semi_transparent.png").to_vec()),
            // ("texture/crate.png".to_string(), include_bytes!("../../../examples/assets/texture/crate.png").to_vec()),
            // ("texture/crate_spritesheet.ron".to_string(), include_bytes!("../../../examples/assets/texture/crate_spritesheet.ron").to_vec()),
            // ("texture/grass.png".to_string(), include_bytes!("../../../examples/assets/texture/grass.png").to_vec()),
            // ("texture/grid.png".to_string(), include_bytes!("../../../examples/assets/texture/grid.png").to_vec()),
            // ("texture/list.png".to_string(), include_bytes!("../../../examples/assets/texture/list.png").to_vec()),
            // ("texture/logo.png".to_string(), include_bytes!("../../../examples/assets/texture/logo.png").to_vec()),
            // ("texture/logo_transparent.png".to_string(), include_bytes!("../../../examples/assets/texture/logo_transparent.png").to_vec()),
            // ("texture/pong_spritesheet.png".to_string(), include_bytes!("../../../examples/assets/texture/pong_spritesheet.png").to_vec()),
            // ("texture/pong_spritesheet.ron".to_string(), include_bytes!("../../../examples/assets/texture/pong_spritesheet.ron").to_vec()),
            // ("texture/test_texture.png".to_string(), include_bytes!("../../../examples/assets/texture/test_texture.png").to_vec()),
            // ("ui/custom.ron".to_string(), include_bytes!("../../../examples/assets/ui/custom.ron").to_vec()),
            // ("ui/example.ron".to_string(), include_bytes!("../../../examples/assets/ui/example.ron").to_vec()),
            // ("ui/fov.ron".to_string(), include_bytes!("../../../examples/assets/ui/fov.ron").to_vec()),
            // ("ui/fps.ron".to_string(), include_bytes!("../../../examples/assets/ui/fps.ron").to_vec()),
            // ("ui/loading.ron".to_string(), include_bytes!("../../../examples/assets/ui/loading.ron").to_vec()),
            // ("ui/paused.ron".to_string(), include_bytes!("../../../examples/assets/ui/paused.ron").to_vec()),
        ].into_iter().collect();

        Inline {
            assets,
        }
    }
}

impl Source for Inline {
    fn modified(&self, path: &str) -> Result<u64, Error> {
        Ok(0)
    }

    fn load(&self, path: &str) -> Result<Vec<u8>, Error> {
        let data = self.assets.get(path).ok_or(Error::from_string(format!("Asset {} not found", path)))?;
        Ok(data.clone())
    }
}
