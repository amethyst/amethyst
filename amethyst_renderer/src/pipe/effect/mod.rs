//! Helper builder for pipeline state objects.

#![allow(missing_docs)]

use self::pso::{Data, Init, Meta};

use error::{Error, Result};
use fnv::FnvHashMap as HashMap;
use gfx::Bind;
use gfx::buffer::{Info as BufferInfo, Role as BufferRole};
use gfx::memory::Usage;
use gfx::{Primitive, ShaderSet};
use gfx::preset::depth::{LESS_EQUAL_TEST, LESS_EQUAL_WRITE};
use gfx::pso::buffer::{ElemStride, InstanceRate};
use gfx::shade::{ProgramError, ToUniform};
use gfx::shade::core::UniformValue;
use gfx::state::{Rasterizer, Stencil};
use gfx::traits::Pod;
use pipe::{Target, Targets};
use scene::Model;
use std::mem;
use types::{Encoder, Factory, PipelineState, Resources};
use vertex::Attribute;

mod pso;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum BlendMode {
    Add,
    Alpha,
    Invert,
    Multiply,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DepthMode {
    LessEqualTest,
    LessEqualWrite,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum ProgramSource<'a> {
    Simple(&'a [u8], &'a [u8]),
    Geometry(&'a [u8], &'a [u8], &'a [u8]),
    Tessellated(&'a [u8], &'a [u8], &'a [u8], &'a [u8]),
}

impl<'a> ProgramSource<'a> {
    pub fn compile(&self, fac: &mut Factory) -> Result<ShaderSet<Resources>> {
        use gfx::Factory;
        use gfx::traits::FactoryExt;

        match *self {
            ProgramSource::Simple(ref vs, ref ps) => {
                fac.create_shader_set(vs, ps)
                    .map_err(|e| Error::ProgramCreation(e))
            }
            ProgramSource::Geometry(ref vs, ref gs, ref ps) => {
                let v = fac.create_shader_vertex(vs)
                    .map_err(|e| ProgramError::Vertex(e))?;
                let g = fac.create_shader_geometry(gs)
                    .expect("Geometry shader creation failed");
                let p = fac.create_shader_pixel(ps)
                    .map_err(|e| ProgramError::Pixel(e))?;
                Ok(ShaderSet::Geometry(v, g, p))
            }
            ProgramSource::Tessellated(ref vs, ref hs, ref ds, ref ps) => {
                fac.create_shader_set_tessellation(vs, hs, ds, ps)
                    .map_err(|e| Error::ProgramCreation(e))
            }
        }
    }
}

#[derive(Derivative)]
#[derivative(Clone, Debug, Eq, PartialEq)]
pub struct Effect {
    pub pso: PipelineState<Meta>,
    pub data: Data,
    const_bufs: HashMap<String, usize>,
    globals: HashMap<String, usize>,
}

impl Effect {
    pub fn update_global<N: AsRef<str>, T: ToUniform>(&mut self, name: N, data: T) {
        if let Some(i) = self.globals.get(name.as_ref()) {
            self.data.globals[*i] = data.convert();
        }
    }

    /// FIXME: Update raw buffer without transmute, use `Result` somehow.
    pub fn update_buffer<N, T>(&self, name: N, data: &[T], enc: &mut Encoder)
        where N: AsRef<str>, T: Pod
    {
        if let Some(i) = self.const_bufs.get(name.as_ref()) {
            let raw = &self.data.const_bufs[*i];
            enc.update_buffer::<T>(unsafe { mem::transmute(raw) }, &data[..], 0);
        }
    }

    /// FIXME: Update raw buffer without transmute.
    pub fn update_constant_buffer<N, T>(&self, name: N, data: &T, enc: &mut Encoder)
        where N: AsRef<str>, T: Copy
    {
        if let Some(i) = self.const_bufs.get(name.as_ref()) {
            let raw = &self.data.const_bufs[*i];
            enc.update_constant_buffer::<T>(unsafe { mem::transmute(raw) }, &data);
        }
    }

    /// FIXME: Add support for arbitrary materials and textures.
    pub fn draw(&self, model: &Model, enc: &mut Encoder) {
        let mut data = self.data.clone();

        let (vbuf, slice) = model.mesh.geometry();
        data.vertex_bufs.push(vbuf.clone());

        enc.draw(&slice, &self.pso, &data);
    }
}

pub struct NewEffect<'f> {
    factory: &'f mut Factory,
    out: &'f Target,
}

impl<'f> NewEffect<'f> {
    pub(crate) fn new(fac: &'f mut Factory, out: &'f Target) -> NewEffect<'f> {
        NewEffect {
            factory: fac,
            out,
        }
    }

    pub fn simple<S: Into<&'f [u8]>>(self, vs: S, ps: S) -> EffectBuilder<'f> {
        let src = ProgramSource::Simple(vs.into(), ps.into());
        EffectBuilder::new(self.factory, self.out, src)
    }

    pub fn geom<S: Into<&'f [u8]>>(self, vs: S, gs: S, ps: S) -> EffectBuilder<'f> {
        let src = ProgramSource::Geometry(vs.into(), gs.into(), ps.into());
        EffectBuilder::new(self.factory, self.out, src)
    }

    pub fn tess<S: Into<&'f [u8]>>(self, vs: S, hs: S, ds: S, ps: S) -> EffectBuilder<'f> {
        let src = ProgramSource::Tessellated(vs.into(), hs.into(), ds.into(), ps.into());
        EffectBuilder::new(self.factory, self.out, src)
    }
}

pub struct EffectBuilder<'a> {
    factory: &'a mut Factory,
    out: &'a Target,
    init: Init<'a>,
    prim: Primitive,
    prog: ProgramSource<'a>,
    rast: Rasterizer,
    const_bufs: Vec<BufferInfo>,
}

impl<'a> EffectBuilder<'a> {
    pub(crate) fn new(fac: &'a mut Factory, out: &'a Target, src: ProgramSource<'a>) -> Self {
        EffectBuilder {
            factory: fac,
            out: out,
            init: Init::default(),
            prim: Primitive::TriangleList,
            rast: Rasterizer::new_fill().with_cull_back(),
            prog: src,
            const_bufs: Vec::new(),
        }
    }

    /// Adds a global constant to this `Effect`.
    pub fn with_raw_global(mut self, name: &'a str) -> Self {
        self.init.globals.push(name);
        self
    }

    /// Adds a raw uniform constant to this `Effect`.
    ///
    /// Requests a new constant buffer to be created
    pub fn with_raw_constant_buffer(mut self, name: &'a str, size: usize, num: usize) -> Self {
        self.const_bufs.push(BufferInfo {
            role: BufferRole::Constant,
            bind: Bind::empty(),
            usage: Usage::Dynamic,
            size: num * size,
            stride: size,
        });
        self.init.const_bufs.push(name);
        self
    }

    /// Sets the output target of the PSO.
    ///
    /// If the target contains a depth buffer, its mode will be set by `depth`.
    pub fn with_output(mut self, name: &'a str, depth: Option<DepthMode>) -> Self {
        if let Some(depth) = depth {
            self.init.out_depth = Some((match depth {
                DepthMode::LessEqualTest => LESS_EQUAL_TEST,
                DepthMode::LessEqualWrite => LESS_EQUAL_WRITE,
            }, Stencil::default()));
        }
        self.init.out_colors.push(name);
        self
    }

    /// Adds a texture sampler to this `Effect`.
    pub fn with_texture(mut self, name: &'a str) -> Self {
        self.init.samplers.push(name);
        self.init.textures.push(name);
        self
    }

    /// Adds a vertex buffer to this `Effect`.
    pub fn with_raw_vertex_buffer(mut self, attrs: &'a [(&'a str, Attribute)], stride: ElemStride, rate: InstanceRate) -> Self {
        self.init.vertex_bufs.push((attrs, stride, rate));
        self
    }

    pub fn build(mut self) -> Result<Effect> {
        use gfx::Factory;
        use gfx::traits::FactoryExt;

        let mut fac = self.factory;
        let prog = self.prog.compile(fac)?;
        let pso = fac.create_pipeline_state(&prog, self.prim, self.rast, self.init.clone())?;

        let mut data = Data::default();

        let const_bufs = self.init.const_bufs
            .iter()
            .enumerate()
            .zip(self.const_bufs.drain(..))
            .map(|((i, name), info)| {
                let cbuf = fac.create_buffer_raw(info)?;
                data.const_bufs.push(cbuf);
                Ok((name.to_string(), i))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let globals = self.init.globals
            .iter()
            .enumerate()
            .map(|(i, name)| {
                // Insert placeholder value until updated by user.
                data.globals.push(UniformValue::F32Vector4([0.0; 4]));
                (name.to_string(), i)
            })
            .collect::<HashMap<_, _>>();

        data.out_colors.extend(self.out.color_bufs().iter().map(|cb| cb.as_output.clone()));
        data.out_depth = self.out.depth_buf().map(|db| (db.as_output.clone(), (0, 0)));

        Ok(Effect {
            pso,
            data,
            const_bufs,
            globals,
        })
    }
}
