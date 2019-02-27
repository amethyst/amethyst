//! Demonstrates how to load and render sprites.
//!
//! Sprites are from <https://opengameart.org/content/bat-32x32>.

use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationSet, AnimationSetPrefab,
        EndControl,
    },
    assets::{AssetStorage, JsonFormat, Loader, PrefabData, PrefabLoaderSystem, ProgressCounter},
    config::Config,
    core::transform::{Transform, TransformBundle},
    derive::PrefabData,
    ecs::{prelude::Entity, Read, ReadExpect, WriteStorage},
    error::Error,
    prelude::{Builder, World},
    renderer::{
        Camera, DisplayConfig, DrawFlat2D, Pipeline, Projection, RenderBundle, ScreenDimensions,
        Sprite, SpriteRender, SpriteSheet, Stage, TextureFormat, TexturePrefab,
    },
    utils::application_root_dir,
    Application, GameData, GameDataBuilder, SimpleState, SimpleTrans, StateData, Trans,
};
use amethyst_assets::PrefabLoader;
use serde::{Deserialize, Serialize};

/// Information about position of the sprite in the sprite sheet
/// Positions originate in the top-left corner (bitmap image convention).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SpritePosition {
    /// Horizontal position of the sprite in the sprite sheet
    pub x: u32,
    /// Vertical position of the sprite in the sprite sheet
    pub y: u32,
    /// Width of the sprite
    pub width: u32,
    /// Height of the sprite
    pub height: u32,
    /// Number of pixels to shift the sprite to the left and down relative to the entity holding it
    pub offsets: Option<[f32; 2]>,
}

/// Represent `amethyst_renderer::SpriteSheetFormat`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SpriteSheetLoader {
    pub spritesheet_width: u32,
    pub spritesheet_height: u32,
    pub sprites: Vec<SpritePosition>,
}

/// `PrefabData` for loading `SpriteRender`.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SpriteRenderPrefab {
    /// Spritesheet texture
    pub texture: TexturePrefab<TextureFormat>,
    /// Spritesheet
    pub sprite_sheet: SpriteSheetLoader,
    /// Index of the default sprite on the sprite sheet
    pub sprite_number: usize,
}

impl<'a> PrefabData<'a> for SpriteRenderPrefab {
    type SystemData = (
        <TexturePrefab<TextureFormat> as PrefabData<'a>>::SystemData,
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<SpriteSheet>>,
        WriteStorage<'a, SpriteRender>,
    );
    type Result = ();

    /// Add loaded SpriteRender to entity
    fn add_to_entity(
        &self,
        entity: Entity,
        (tex_data, loader, sheet_storage, render_storage): &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        // Creates a Sprite from pixel values
        let mut sprites: Vec<Sprite> = Vec::with_capacity(self.sprite_sheet.sprites.len());
        for sp in &self.sprite_sheet.sprites {
            let sprite = Sprite::from_pixel_values(
                self.sprite_sheet.spritesheet_width as u32,
                self.sprite_sheet.spritesheet_height as u32,
                sp.width as u32,
                sp.height as u32,
                sp.x as u32,
                sp.y as u32,
                sp.offsets.unwrap_or([0.0; 2]),
            );
            sprites.push(sprite);
        }

        // Get texture handle from TexturePrefab
        let texture = self.texture.add_to_entity(entity, tex_data, entities)?;

        // Load SpriteSheet and get handle
        let sheet = SpriteSheet { texture, sprites };
        let sheet_handle = loader.load_from_data(sheet, (), sheet_storage);

        // Add SpriteRender to entity
        let render = SpriteRender {
            sprite_sheet: sheet_handle,
            sprite_number: self.sprite_number,
        };
        render_storage.insert(entity, render)?;

        Ok(())
    }

    /// Load TexturePrefab
    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        (tex_data, _, _, _): &mut Self::SystemData,
    ) -> Result<bool, Error> {
        self.texture.load_sub_assets(progress, tex_data)
    }
}

/// Animation ids used in a AnimationSet
#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Fly,
}

/// Loading data for one entity
#[derive(Debug, Clone, Serialize, Deserialize, PrefabData)]
struct MyPrefabData {
    /// Rendering position and orientation
    transform: Transform,
    /// Information for rendering a sprite
    sprite_render: SpriteRenderPrefab,
    /// Аll animations that can be run on the entity
    animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
}

/// The main state
#[derive(Default)]
struct Example {
    /// A progress tracker to check that assets are loaded
    pub progress_counter: Option<ProgressCounter>,
    /// Bat entity to start animation after loading
    pub bat: Option<Entity>,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        // Crates new progress counter
        self.progress_counter = Some(Default::default());
        // Starts asset loading
        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load(
                // FIXME: deserialization of untagged enum in `ron` is buggy
                "prefab/sprite_animation.json",
                JsonFormat,
                (),
                self.progress_counter.as_mut().unwrap(),
            )
        });
        // Creates a new entity with components from MyPrefabData
        self.bat = Some(world.create_entity().with(prefab_handle).build());
        // Creates a new camera
        initialise_camera(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // Checks if we are still loading data
        if let Some(ref progress_counter) = self.progress_counter {
            // Checks progress
            if progress_counter.is_complete() {
                let StateData { world, .. } = data;
                // Gets the Fly animation from AnimationSet
                let animation = world
                    .read_storage::<AnimationSet<AnimationId, SpriteRender>>()
                    .get(self.bat.unwrap())
                    .and_then(|s| s.get(&AnimationId::Fly).cloned())
                    .unwrap();
                // Creates a new AnimationControlSet for bat entity
                let mut sets = world.write_storage();
                let control_set =
                    get_animation_set::<AnimationId, SpriteRender>(&mut sets, self.bat.unwrap())
                        .unwrap();
                // Adds the animation to AnimationControlSet and loops infinitely
                control_set.add_animation(
                    AnimationId::Fly,
                    &animation,
                    EndControl::Loop(None),
                    1.0,
                    AnimationCommand::Start,
                );
                // All data loaded
                self.progress_counter = None;
            }
        }
        Trans::None
    }
}

fn initialise_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };

    let mut camera_transform = Transform::default();
    camera_transform.set_z(1.0);

    world
        .create_entity()
        .with(camera_transform)
        .with(Camera::from(Projection::orthographic(
            0.0, width, 0.0, height,
        )))
        .build();
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("examples/assets/");
    let display_conf_path = app_root.join("examples/sprite_animation/resources/display_config.ron");
    let display_config = DisplayConfig::load(display_conf_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::new()),
    );

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(AnimationBundle::<AnimationId, SpriteRender>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(RenderBundle::new(pipe, Some(display_config)).with_sprite_sheet_processor())?;

    let mut game = Application::new(assets_directory, Example::default(), game_data)?;
    game.run();

    Ok(())
}
