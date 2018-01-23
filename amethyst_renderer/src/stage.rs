//!
//!
//!
//!
use gfx_hal::pso::{ShaderStageFlags, Stage};

pub trait ShaderStage {
    fn flags() -> ShaderStageFlags;
}

pub enum Vertex {}
impl ShaderStage for Vertex {
    #[inline(always)]
    fn flags() -> ShaderStageFlags {
        ShaderStageFlags::VERTEX
    }
}

pub enum Hull {}
impl ShaderStage for Hull {
    #[inline(always)]
    fn flags() -> ShaderStageFlags {
        ShaderStageFlags::HULL
    }
}

pub enum Domain {}
impl ShaderStage for Domain {
    #[inline(always)]
    fn flags() -> ShaderStageFlags {
        ShaderStageFlags::DOMAIN
    }
}

pub enum Geometry {}
impl ShaderStage for Geometry {
    #[inline(always)]
    fn flags() -> ShaderStageFlags {
        ShaderStageFlags::GEOMETRY
    }
}

pub enum Fragment {}
impl ShaderStage for Fragment {
    #[inline(always)]
    fn flags() -> ShaderStageFlags {
        ShaderStageFlags::FRAGMENT
    }
}

pub enum Compute {}
impl ShaderStage for Compute {
    #[inline(always)]
    fn flags() -> ShaderStageFlags {
        ShaderStageFlags::COMPUTE
    }
}

macro_rules! impl_shader_stage {
    (~ $h:ident $(,$t:ident)*) => {
        impl_shader_stage!($($t),*);
    };
    ($($a:ident),+) => {
        impl_shader_stage!(~ $($a),+);

        impl<$($a),+> ShaderStage for ($($a,)+)
        where
            $($a: ShaderStage),+
        {
            #[inline(always)]
            fn flags() -> ShaderStageFlags {
                $($a::flags())|+
            }
        }
    };
    () => {};
}

impl_shader_stage!(A, B, C, D, E, F, G);
