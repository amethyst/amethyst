//! GPU POD data types.
use crate::{
    mtl,
    resources::Tint as TintComponent,
    sprite::{SpriteRender, SpriteSheet},
    types::Texture,
    light,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    math::{convert, Matrix4, Vector3, Vector4, Point3, RealField, U1, U3, Translation3},
    Transform
};
use glsl_layout::*;
use rendy::{
    hal::format::Format,
    mesh::{AsAttribute, AsVertex, Model, VertexFormat},
};
use lazy_static::lazy_static;
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
            tint: tint.map_or([1.0; 4].into(), |t| t.0.into_pod()),
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
            tint: tint.map_or([1.0; 4].into(), |t| t.0.into_pod()),
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
///    float radius;
///    float smoothness;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
pub struct PointLight {
    /// Light world position
    pub position: vec3,
    /// Light color
    pub color: vec3,
    /// Luminous intensity (cd)
    pub intensity: float,
    // Pointlight radius
    pub radius: float,
    /// Pointlight smoothness
    pub smoothness: float,
}
impl From<(&Transform, &light::punctual::PointLight)> for PointLight {
    fn from((transform, light): (&Transform, &light::punctual::PointLight)) -> PointLight {
        PointLight {
            position: convert::<_, Vector3<f32>>(
                transform.global_matrix().column(3).xyz(),
            )
            .into_pod(),
            color: light.color.into_pod(),
            intensity: light.intensity / (4.0 * f32::pi()),
            radius: light.radius,
            smoothness: light.smoothness,
        }
    }
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
    /// Luminous intensity (cd)
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
    /// Luminous intensity (cd)
    pub intensity: float,
    /// Spotlight range
    pub range: float,
    /// Spotlight smoothness
    pub smoothness: float,
}

impl From<(&Transform, &light::punctual::SpotLight)> for SpotLight {
    fn from((transform, light): (&Transform, &light::punctual::SpotLight)) -> SpotLight {
        SpotLight {
            position: convert::<_, Vector3<f32>>(
                transform.global_matrix().column(3).xyz(),
            )
            .into_pod(),
            color: light.color.into_pod(),
            direction: light.direction.into_pod(),
            angle: light.angle.cos(),
            intensity: light.intensity / f32::pi(),
            range: light.range,
            smoothness: light.smoothness,
        }
    }
}

lazy_static!{
    static ref LIGHT_VERTICES: [Point3<f32>; 6] = [
        Point3::new(-0.5, -0.5, 0.0),
        Point3::new(0.5, -0.5, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(-0.5, 0.5, 0.0),
        Point3::new(0.0, -0.5, 0.0),
        Point3::new(0.0, 0.5, 0.0),
    ];
}

/// area light struct
/// ```glsl,ignore
/// struct RoundAreaLight {
///    vec3 position;
///    vec3 color;
///    float intensity;
///    bool two_sided;
///    bool sphere;
///    vec3[4] quad_points;
/// };
/// ```
// todo: Merge some of the fields for optimization
#[derive(Clone, Copy, Debug, AsStd140)]
pub struct AreaLight {
    /// Light world position
    pub position: vec3,
    /// Color of the diffuse part of the light.
    pub diffuse_color: vec3,
    /// Color of the specular part of the light.
    pub spec_color: vec3,
    /// Intensity in nits.
    pub intensity: float,
    /// Wether the light lights behind as well.
    pub two_sided: boolean,
    /// Wether the light is a sphere.
    pub sphere: boolean,
    /// Point for integrating the area.
    pub quad_points: [vec3; 4],
}

impl From<(&Transform, &light::area::Sphere)> for AreaLight {
    fn from((transform, light): (&Transform, &light::area::Sphere)) -> AreaLight {
        let transform = transform.global_matrix();
        let translation = Translation3::from(transform.fixed_slice::<U3, U1>(0, 3).xyz());
        let scale_x = transform.fixed_slice::<U3, U1>(0, 0).norm();
        let scale_y = transform.fixed_slice::<U3, U1>(0, 1).norm();
        AreaLight {
            position: transform.column(3).xyz().into_pod(),
            diffuse_color: light.diffuse_color.into_pod(),
            spec_color: light.spec_color.into_pod(),
            intensity: light.intensity.luminance_or(|x| { x / (4.0 * scale_x * scale_y * f32::pi() * f32::pi() )}),
            two_sided: true.into(),
            sphere: true.into(),
            quad_points: [
                translation.transform_point(&LIGHT_VERTICES[0]).into_pod(),
                translation.transform_point(&LIGHT_VERTICES[1]).into_pod(),
                translation.transform_point(&LIGHT_VERTICES[2]).into_pod(),
                translation.transform_point(&LIGHT_VERTICES[3]).into_pod(),
            ]
        }
    }
}

impl From<(&Transform, &light::area::Disk)> for AreaLight {
    fn from((transform, light): (&Transform, &light::area::Disk)) -> AreaLight {
        let transform = transform.global_matrix();
        let scale_x = transform.fixed_slice::<U3, U1>(0, 0).norm();
        let scale_y = transform.fixed_slice::<U3, U1>(0, 1).norm();
        AreaLight {
            position: transform.column(3).xyz().into_pod(),
            diffuse_color: light.diffuse_color.into_pod(),
            spec_color: light.spec_color.into_pod(),
            intensity: light.intensity.luminance_or(|x| { x / (scale_x * scale_y * f32::pi() * f32::pi() )}),
            two_sided: light.two_sided.into(),
            sphere: false.into(),
            quad_points: [
                transform.transform_point(&LIGHT_VERTICES[0]).into_pod(),
                transform.transform_point(&LIGHT_VERTICES[1]).into_pod(),
                transform.transform_point(&LIGHT_VERTICES[2]).into_pod(),
                transform.transform_point(&LIGHT_VERTICES[3]).into_pod(),
            ]
        }
    }
}

impl From<(&Transform, &light::area::Rectangle)> for AreaLight {
    fn from((transform, light): (&Transform, &light::area::Rectangle)) -> AreaLight {
        let transform = transform.global_matrix();
        let scale_x = transform.fixed_slice::<U3, U1>(0, 0).norm();
        let scale_y = transform.fixed_slice::<U3, U1>(0, 1).norm();
        AreaLight {
            position: transform.column(3).xyz().into_pod(),
            diffuse_color: light.diffuse_color.into_pod(),
            spec_color: light.spec_color.into_pod(),
            intensity: light.intensity.luminance_or(|x| { x / (scale_x * scale_y )}),
            two_sided: light.two_sided.into(),
            sphere: false.into(),
            quad_points: [
                transform.transform_point(&LIGHT_VERTICES[0]).into_pod(),
                transform.transform_point(&LIGHT_VERTICES[1]).into_pod(),
                transform.transform_point(&LIGHT_VERTICES[2]).into_pod(),
                transform.transform_point(&LIGHT_VERTICES[3]).into_pod(),
            ]
        }
    }
}

/// area light struct
/// ```glsl,ignore
/// struct TubularAreaLight {
///    vec3 position;
///    vec3 diffuse_color;
///    vec3 spec_color;
///    float intensity;
///    float radius;
///    vec3 quad_points[2];
///    bool end_caps;
/// };
/// ```
// todo: Merge some of the fields for optimization
#[derive(Clone, Copy, Debug, AsStd140)]
pub struct TubularAreaLight {
    /// Light world position
    pub position: vec3,
    /// Light Color
    pub diffuse_color: vec3,
    pub spec_color: vec3,
    pub intensity: float,
    pub radius: float,
    pub quad_points: [vec3; 2],
    pub end_caps: boolean,
}

impl From<(&Transform, &light::area::Tube)> for TubularAreaLight {
    fn from((transform, light): (&Transform, &light::area::Tube)) -> TubularAreaLight {
        let transform = transform.global_matrix();
        let scale_x = transform.fixed_slice::<U3, U1>(0, 0).norm();
        let scale_y = transform.fixed_slice::<U3, U1>(0, 1).norm();
        TubularAreaLight {
            position: transform.column(3).xyz().into_pod(),
            diffuse_color: light.diffuse_color.into_pod(),
            spec_color: light.spec_color.into_pod(),
            intensity: light.intensity.luminance_or(|x| { x / (scale_x * scale_y )}),
            radius: light.radius.into(),
            quad_points: [
                transform.transform_point(&LIGHT_VERTICES[4]).into_pod(),
                transform.transform_point(&LIGHT_VERTICES[5]).into_pod(),
            ],
            end_caps: light.end_caps.into(),
        }
    }
}



/// Environment Uniform
/// ```glsl,ignore
/// uniform Environment {
///    vec3 ambient_color;
///    vec3 camera_position;
///    int point_light_count;
///    int directional_light_count;
///    int spot_light_count;
///    int round_area_light_count;
///    int rect_area_light_count;
///    int line_area_light_count;
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
    /// Number of round area lights
    pub round_area_light_count: int,
    /// Number of rectangular area lights
    pub rect_area_light_count: int,
    /// Number of tubular area lights
    pub line_area_light_count: int,
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
                    let (r, g, b, a) = t.0.into_components();
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

impl IntoPod<vec3> for amethyst_core::math::Point3<f32> {
    fn into_pod(self) -> vec3 {
        let arr: [f32; 3] = self.coords.into();
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
