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

#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(16))]
pub struct TextureOffset {
    pub u_offset: vec2,
    pub v_offset: vec2,
}

impl TextureOffset {
    pub fn from_offset(offset: &crate::mtl::TextureOffset) -> Self {
        TextureOffset {
            u_offset: [offset.u.0, offset.u.1].into(),
            v_offset: [offset.v.0, offset.v.1].into(),
        }
    }
}

#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(16))]
pub struct ViewArgs {
    pub proj: mat4,
    pub view: mat4,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(16))]
pub struct Tint {
    pub tint: vec4,
}

impl AsAttribute for Tint {
    const NAME: &'static str = "tint";
    const FORMAT: Format = Format::Rgba32Sfloat;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct VertexArgs {
    pub model: mat4,
    pub tint: vec4,
}

impl VertexArgs {
    #[inline]
    pub fn from_object_data(transform: &Transform, tint: Option<&TintComponent>) -> Self {
        let model: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(*transform.global_matrix()).into();
        VertexArgs {
            model: model.into(),
            tint: tint.map_or([1.0; 4].into(), |t| {
                let (r, g, b, a) = t.0.into_components();
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct JointsOffset {
    pub joints_offset: u32,
}

impl AsAttribute for JointsOffset {
    const NAME: &'static str = "joints_offset";
    const FORMAT: Format = Format::R32Uint;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct SkinnedVertexArgs {
    pub model: mat4,
    pub tint: vec4,
    pub joints_offset: u32,
}

impl AsVertex for SkinnedVertexArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((Model::vertex(), Tint::vertex(), JointsOffset::vertex()))
    }
}

impl SkinnedVertexArgs {
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
                let (r, g, b, a) = t.0.into_components();
                [r, g, b, a].into()
            }),
            joints_offset,
        }
    }
}

#[derive(Clone, Copy, Debug, AsStd140)]
pub struct PointLight {
    pub position: vec3,
    pub color: vec3,
    pub intensity: float,
}

#[derive(Clone, Copy, Debug, AsStd140)]
pub struct DirectionalLight {
    pub color: vec3,
    pub intensity: float,
    pub direction: vec3,
}

#[derive(Clone, Copy, Debug, AsStd140)]
pub struct SpotLight {
    pub position: vec3,
    pub color: vec3,
    pub direction: vec3,
    pub angle: float,
    pub intensity: float,
    pub range: float,
    pub smoothness: float,
}

#[derive(Clone, Copy, Debug, AsStd140)]
pub struct Environment {
    pub ambient_color: vec3,
    pub camera_position: vec3,
    pub point_light_count: int,
    pub directional_light_count: int,
    pub spot_light_count: int,
}

#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(16))]
pub struct Material {
    pub uv_offset: TextureOffset,
    pub alpha_cutoff: float,
}

impl Material {
    pub fn from_material(mat: &mtl::Material) -> Self {
        Material {
            uv_offset: TextureOffset::from_offset(&mat.uv_offset),
            alpha_cutoff: mat.alpha_cutoff,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct SpriteArgs {
    pub dir_x: vec2,
    pub dir_y: vec2,
    pub pos: vec2,
    pub u_offset: vec2,
    pub v_offset: vec2,
    pub depth: float,
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
        ))
    }
}

impl SpriteArgs {
    pub fn from_data<'a>(
        tex_storage: &AssetStorage<Texture>,
        sprite_storage: &'a AssetStorage<SpriteSheet>,
        sprite_render: &SpriteRender,
        transform: &Transform,
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
            },
            &sprite_sheet.texture,
        ))
    }
}

pub trait IntoPod<T> {
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
