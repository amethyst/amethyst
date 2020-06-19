//! GPU POD data types.
use crate::{
    mtl,
    resources::Tint as TintComponent,
    sprite::{SpriteRender, SpriteSheet},
    types::Texture,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    math::{convert, Matrix4, Vector4},
    Transform,
};
use glsl_layout::*;
use rendy::{
    hal::format::Format,
    mesh::{AsAttribute, AsVertex, Model, VertexFormat},
};

/// TextureOffset
/// ```glsl,ignore
/// struct UvOffset {
///    vec2 u_offset;
///    vec2 v_offset;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(16))]
pub struct TextureOffset {
    /// U-axis offset
    pub u_offset: vec2,
    /// V-axis offset
    pub v_offset: vec2,
}

impl TextureOffset {
    /// Helper function from proper type to Pod type.
    pub fn from_offset(offset: &crate::mtl::TextureOffset) -> Self {
        TextureOffset {
            u_offset: [offset.u.0, offset.u.1].into(),
            v_offset: [offset.v.0, offset.v.1].into(),
        }
    }
}

/// ViewArgs
/// ```glsl,ignore
/// uniform ViewArgs {
///    uniform mat4 proj;
///    uniform mat4 view;
///    uniform mat4 proj_view;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(16))]
pub struct ViewArgs {
    /// Projection matrix
    pub proj: mat4,
    /// View matrix
    pub view: mat4,
    /// Premultiplied Proj-View matrix
    pub proj_view: mat4,
}

/// Tint
/// ```glsl,ignore
/// vec4 tint;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(16))]
pub struct Tint {
    /// Tint color as `Rgba32Sfloat`
    pub tint: vec4,
}

impl AsAttribute for Tint {
    const NAME: &'static str = "tint";
    const FORMAT: Format = Format::Rgba32Sfloat;
}

/// Instance-rate vertex arguments
/// ```glsl,ignore
///  mat4 model;
///  vec4 tint;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct VertexArgs {
    /// Instance-rate model matrix
    pub model: mat4,
    /// Instance-rate model `Tint`
    pub tint: vec4,
}

impl VertexArgs {
    /// Populates a `VertexArgs` instance-rate structure with the information from a `Transform`
    /// and `TintComponent` components.
    #[inline]
    pub fn from_object_data(transform: &Transform, tint: Option<&TintComponent>) -> Self {
        let model: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(*transform.global_matrix()).into();
        VertexArgs {
            model: model.into(),
            tint: tint.map_or([1.0; 4].into(), |t| {
                // Shaders expect linear RGBA; convert sRGBA to linear RGBA
                let (r, g, b, a) = t.0.into_linear().into_components();
                [r, g, b, a].into()
            }),
        }
    }
}

impl AsVertex for VertexArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((Model::vertex(), Tint::vertex()))
    }
}

/// Instance-rate joints offset
/// ```glsl,ignore
///  uint joints_offset;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct JointsOffset {
    /// `u32` joints offset value
    pub joints_offset: u32,
}

impl AsAttribute for JointsOffset {
    const NAME: &'static str = "joints_offset";
    const FORMAT: Format = Format::R32Uint;
}

/// Skinned Instance-rate vertex arguments.
/// ```glsl,ignore
///  mat4 model;
///  vec4 tint;
///  uint joints_offset:
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct SkinnedVertexArgs {
    /// Instance-rate model matrix
    pub model: mat4,
    /// Instance-rate `Tint`
    pub tint: vec4,
    /// Instance-rate joint offset as `u32`
    pub joints_offset: u32,
}

impl AsVertex for SkinnedVertexArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((Model::vertex(), Tint::vertex(), JointsOffset::vertex()))
    }
}

impl SkinnedVertexArgs {
    /// Populate `SkinnedVertexArgs` from the supplied `Transform` and `TintComponent`
    #[inline]
    pub fn from_object_data(
        transform: &Transform,
        tint: Option<&TintComponent>,
        joints_offset: u32,
    ) -> Self {
        let model: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(*transform.global_matrix()).into();
        SkinnedVertexArgs {
            model: model.into(),
            tint: tint.map_or([1.0; 4].into(), |t| {
                // Shaders expect linear RGBA; convert sRGBA to linear RGBA
                let (r, g, b, a) = t.0.into_linear().into_components();
                [r, g, b, a].into()
            }),
            joints_offset,
        }
    }
}

/// point light struct
/// ```glsl,ignore
/// struct PointLight {
///    vec3 position;
///    vec3 color;
///    float intensity;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
pub struct PointLight {
    /// Light world position
    pub position: vec3,
    /// Light color
    pub color: vec3,
    /// Light intensity (0 - infinity)
    pub intensity: float,
}

/// directional light struct
/// ```glsl,ignore
/// struct DirectionalLight {
///    vec3 color;
///    float intensity;
///    vec3 direction;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
pub struct DirectionalLight {
    /// Light Color
    pub color: vec3,
    /// Light intensity (0 - infinity)
    pub intensity: float,
    /// light cast direction vector
    pub direction: vec3,
}

/// spot light struct
/// ```glsl,ignore
/// struct SpotLight {
///    vec3 position;
///    vec3 color;
///    vec3 direction;
///    float angle;
///    float intensity;
///    float range;
///    float smoothness;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
pub struct SpotLight {
    /// Light world position
    pub position: vec3,
    /// Light Color
    pub color: vec3,
    /// Light direction
    pub direction: vec3,
    /// Angle of the light in radians
    pub angle: float,
    /// Light intensity (0 - infinity)
    pub intensity: float,
    /// Spotlight range
    pub range: float,
    /// Spotlight smoothness
    pub smoothness: float,
}

/// Environment Uniform
/// ```glsl,ignore
/// uniform Environment {
///    vec3 ambient_color;
///    vec3 camera_position;
///    int point_light_count;
///    int directional_light_count;
///    int spot_light_count;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
pub struct Environment {
    /// Ambient color for the entire image
    pub ambient_color: vec3,
    /// Camera world position
    pub camera_position: vec3,
    /// Number of point lights
    pub point_light_count: int,
    /// Number of directional lights
    pub directional_light_count: int,
    /// Number of spot lights
    pub spot_light_count: int,
}

/// Material Uniform
/// ```glsl,ignore
/// uniform Material {
///    UvOffset uv_offset;
///    float alpha_cutoff;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(16))]
pub struct Material {
    /// UV offset of material
    pub uv_offset: TextureOffset,
    /// Material alpha cutoff
    pub alpha_cutoff: float,
}

impl Material {
    /// Helper function from amethyst_rendy 'proper' type to POD type.
    pub fn from_material(mat: &mtl::Material) -> Self {
        Material {
            uv_offset: TextureOffset::from_offset(&mat.uv_offset),
            alpha_cutoff: mat.alpha_cutoff,
        }
    }
}

/// Sprite Vertex Data
/// ```glsl,ignore
/// vec2 dir_x;
/// vec2 dir_y;
/// vec2 pos;
/// vec2 u_offset;
/// vec2 v_offset;
/// float depth;
/// vec4 tint;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct SpriteArgs {
    /// Rotation of the sprite, X-axis
    pub dir_x: vec2,
    /// Rotation of the sprite, Y-axis
    pub dir_y: vec2,
    /// Screen position of the sprite
    pub pos: vec2,
    /// Upper-left coordinate of the sprite in the spritesheet
    pub u_offset: vec2,
    /// Bottom-right coordinate of the sprite in the spritesheet
    pub v_offset: vec2,
    /// Depth value of this sprite
    pub depth: float,
    /// Tint for this this sprite
    pub tint: vec4,
}

impl AsVertex for SpriteArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "dir_x"),
            (Format::Rg32Sfloat, "dir_y"),
            (Format::Rg32Sfloat, "pos"),
            (Format::Rg32Sfloat, "u_offset"),
            (Format::Rg32Sfloat, "v_offset"),
            (Format::R32Sfloat, "depth"),
            (Format::Rgba32Sfloat, "tint"),
        ))
    }
}

impl SpriteArgs {
    /// Extracts POD vertex data from the provided storages for a sprite.
    ///
    /// # Arguments
    /// * `tex_storage` - `Texture` Storage
    /// * `sprite_storage` - `SpriteSheet` Storage
    /// * `sprite_render` - `SpriteRender` component reference
    /// * `transform` - 'Transform' component reference
    pub fn from_data<'a>(
        tex_storage: &AssetStorage<Texture>,
        sprite_storage: &'a AssetStorage<SpriteSheet>,
        sprite_render: &SpriteRender,
        transform: &Transform,
        tint: Option<&TintComponent>,
    ) -> Option<(Self, &'a Handle<Texture>)> {
        let sprite_sheet = sprite_storage.get(&sprite_render.sprite_sheet)?;
        if !tex_storage.contains(&sprite_sheet.texture) {
            return None;
        }

        let sprite = &sprite_sheet.sprites[sprite_render.sprite_number];

        let transform = convert::<_, Matrix4<f32>>(*transform.global_matrix());
        let dir_x = transform.column(0) * sprite.width;
        let dir_y = transform.column(1) * -sprite.height;
        let pos = transform * Vector4::new(-sprite.offsets[0], -sprite.offsets[1], 0.0, 1.0);

        Some((
            SpriteArgs {
                dir_x: dir_x.xy().into_pod(),
                dir_y: dir_y.xy().into_pod(),
                pos: pos.xy().into_pod(),
                u_offset: [sprite.tex_coords.left, sprite.tex_coords.right].into(),
                v_offset: [sprite.tex_coords.top, sprite.tex_coords.bottom].into(),
                depth: pos.z,
                tint: tint.map_or([1.0; 4].into(), |t| {
                    // Shaders expect linear RGBA; convert sRGBA to linear RGBA
                    let (r, g, b, a) = t.0.into_linear().into_components();
                    [r, g, b, a].into()
                }),
            },
            &sprite_sheet.texture,
        ))
    }
}

/// Trait for auto conversion into standard GLSL POD types.
pub trait IntoPod<T> {
    /// Converts `Self` to the supplied `T` GLSL type.
    fn into_pod(self) -> T;
}

impl IntoPod<vec2> for amethyst_core::math::Vector2<f32> {
    fn into_pod(self) -> vec2 {
        let arr: [f32; 2] = self.into();
        arr.into()
    }
}

impl IntoPod<vec3> for amethyst_core::math::Vector3<f32> {
    fn into_pod(self) -> vec3 {
        let arr: [f32; 3] = self.into();
        arr.into()
    }
}

impl IntoPod<vec3> for palette::Srgb {
    fn into_pod(self) -> vec3 {
        let (r, g, b) = self.into_components();
        [r, g, b].into()
    }
}

impl IntoPod<[f32; 3]> for palette::Srgb {
    fn into_pod(self) -> [f32; 3] {
        let (r, g, b) = self.into_components();
        [r, g, b]
    }
}

impl IntoPod<vec4> for palette::Srgba {
    fn into_pod(self) -> vec4 {
        let (r, g, b, a) = self.into_components();
        [r, g, b, a].into()
    }
}

impl IntoPod<[f32; 4]> for palette::Srgba {
    fn into_pod(self) -> [f32; 4] {
        let (r, g, b, a) = self.into_components();
        [r, g, b, a]
    }
}
