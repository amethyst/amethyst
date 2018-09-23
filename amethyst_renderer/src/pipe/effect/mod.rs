//! Helper builder for pipeline state objects.

#![allow(missing_docs)]

pub use self::pso::{Data, Init, Meta};
use error::{Error, Result};
use fnv::FnvHashMap as HashMap;
use gfx::buffer::{Info as BufferInfo, Role as BufferRole};
use gfx::memory::{Bind, Usage};
use gfx::preset::depth::{LESS_EQUAL_TEST, LESS_EQUAL_WRITE};
use gfx::pso::buffer::{ElemStride, InstanceRate};
use gfx::shade::core::UniformValue;
use gfx::shade::{ProgramError, ToUniform};
use gfx::state::{Blend, ColorMask, Comparison, CullFace, Depth, MultiSample, Rasterizer, Stencil};
use gfx::traits::Pod;
use gfx::{Primitive, ShaderSet};
use glsl_layout::Std140;
use pipe::Target;
use std::mem;
use types::{Encoder, Factory, PipelineState, Resources, Slice};
use vertex::Attributes;

mod pso;

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
        use gfx::traits::FactoryExt;
        use gfx::Factory;

        match *self {
            ProgramSource::Simple(ref vs, ref ps) => fac
                .create_shader_set(vs, ps)
                .map_err(|e| Error::ProgramCreation(e)),
            ProgramSource::Geometry(ref vs, ref gs, ref ps) => {
                let v = fac
                    .create_shader_vertex(vs)
                    .map_err(|e| ProgramError::Vertex(e))?;
                let g = fac
                    .create_shader_geometry(gs)
                    .expect("Geometry shader creation failed");
                let p = fac
                    .create_shader_pixel(ps)
                    .map_err(|e| ProgramError::Pixel(e))?;
                Ok(ShaderSet::Geometry(v, g, p))
            }
            ProgramSource::Tessellated(ref vs, ref hs, ref ds, ref ps) => fac
                .create_shader_set_tessellation(vs, hs, ds, ps)
                .map_err(|e| Error::ProgramCreation(e)),
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
        match self.globals.get(name.as_ref()) {
            Some(i) => self.data.globals[*i] = data.convert(),
            None => {
                warn!(
                    "Global update for effect failed! Global not found: {:?}",
                    name.as_ref()
                );
            }
        }
    }

    /// FIXME: Update raw buffer without transmute, use `Result` somehow.
    pub fn update_buffer<N, T>(&mut self, name: N, data: &[T], enc: &mut Encoder)
    where
        N: AsRef<str>,
        T: Pod,
    {
        match self.const_bufs.get(name.as_ref()) {
            Some(i) => {
                let raw = &self.data.const_bufs[*i];
                enc.update_buffer::<T>(unsafe { mem::transmute(raw) }, &data[..], 0)
                    .expect("Failed to update buffer (TODO: replace expect)");
            }
            None => {
                warn!(
                    "Buffer update for effect failed! Buffer not found: {:?}",
                    name.as_ref()
                );
            }
        }
    }

    /// FIXME: Update raw buffer without transmute.
    pub fn update_constant_buffer<N, T>(&mut self, name: N, data: &T, enc: &mut Encoder)
    where
        N: AsRef<str>,
        T: Std140,
    {
        match self.const_bufs.get(name.as_ref()) {
            Some(i) => {
                let raw = &self.data.const_bufs[*i];
                enc.update_constant_buffer::<T>(unsafe { mem::transmute(raw) }, &data)
            }
            None => {
                warn!(
                    "Buffer update for effect failed! Buffer not found: {:?}",
                    name.as_ref()
                );
            }
        }
    }

    pub fn clear(&mut self) {
        self.data.textures.clear();
        self.data.samplers.clear();
        self.data.vertex_bufs.clear();
    }

    pub fn draw(&mut self, slice: &Slice, enc: &mut Encoder) {
        enc.draw(&slice, &self.pso, &self.data);
    }
}

pub struct NewEffect<'f> {
    pub factory: &'f mut Factory,
    out: &'f Target,
    multisampling: u16,
}

impl<'f> NewEffect<'f> {
    pub(crate) fn new(fac: &'f mut Factory, out: &'f Target, multisampling: u16) -> Self {
        NewEffect {
            factory: fac,
            out,
            multisampling,
        }
    }

    pub fn simple<S: Into<&'f [u8]>>(self, vs: S, ps: S) -> EffectBuilder<'f> {
        let src = ProgramSource::Simple(vs.into(), ps.into());
        EffectBuilder::new(self.factory, self.out, self.multisampling, src)
    }

    pub fn geom<S: Into<&'f [u8]>>(self, vs: S, gs: S, ps: S) -> EffectBuilder<'f> {
        let src = ProgramSource::Geometry(vs.into(), gs.into(), ps.into());
        EffectBuilder::new(self.factory, self.out, self.multisampling, src)
    }

    pub fn tess<S: Into<&'f [u8]>>(self, vs: S, hs: S, ds: S, ps: S) -> EffectBuilder<'f> {
        let src = ProgramSource::Tessellated(vs.into(), hs.into(), ds.into(), ps.into());
        EffectBuilder::new(self.factory, self.out, self.multisampling, src)
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
    pub(crate) fn new(
        fac: &'a mut Factory,
        out: &'a Target,
        multisampling: u16,
        src: ProgramSource<'a>,
    ) -> Self {
        let mut rast = Rasterizer::new_fill().with_cull_back();
        if multisampling > 0 {
            rast.samples = Some(MultiSample);
        }
        EffectBuilder {
            factory: fac,
            out: out,
            init: Init::default(),
            prim: Primitive::TriangleList,
            rast,
            prog: src,
            const_bufs: Vec::new(),
        }
    }

    /// Disable back face culling
    pub fn without_back_face_culling(&mut self) -> &mut Self {
        self.rast.cull_face = CullFace::Nothing;
        self
    }

    /// Adds a global constant to this `Effect`.
    pub fn with_raw_global(&mut self, name: &'a str) -> &mut Self {
        self.init.globals.push(name);
        self
    }

    /// Adds a raw uniform constant to this `Effect`.
    ///
    /// Requests a new constant buffer to be created
    pub fn with_raw_constant_buffer(
        &mut self,
        name: &'a str,
        size: usize,
        num: usize,
    ) -> &mut Self {
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

    /// Set the pipeline primitive type.
    pub fn with_primitive_type(&mut self, prim: Primitive) -> &mut Self {
        self.prim = prim;
        self
    }

    /// Sets the output target of the PSO.
    ///
    /// If the target contains a depth buffer, its mode will be set by `depth`.
    pub fn with_output(&mut self, name: &'a str, depth: Option<DepthMode>) -> &mut Self {
        if let Some(depth) = depth {
            self.init.out_depth = Some((
                match depth {
                    DepthMode::LessEqualTest => LESS_EQUAL_TEST,
                    DepthMode::LessEqualWrite => LESS_EQUAL_WRITE,
                },
                Stencil::default(),
            ));
        }
        // OSX doesn't seem to work without a depth test, so here's a workaround.
        if cfg!(target_os = "macos") && depth.is_none() {
            self.init.out_depth = Some((
                Depth {
                    fun: Comparison::Always,
                    write: true,
                },
                Stencil::default(),
            ));
        }
        self.init.out_colors.push(name);
        self
    }

    /// Sets the output target of the PSO.
    ///
    /// If the target contains a depth buffer, its mode will be set by `depth`.
    pub fn with_blended_output(
        &mut self,
        name: &'a str,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> &mut Self {
        if let Some(depth) = depth {
            self.init.out_depth = Some((
                match depth {
                    DepthMode::LessEqualTest => LESS_EQUAL_TEST,
                    DepthMode::LessEqualWrite => LESS_EQUAL_WRITE,
                },
                Stencil::default(),
            ));
        }
        // OSX doesn't seem to work without a depth test, so here's a workaround.
        if cfg!(target_os = "macos") && depth.is_none() {
            self.init.out_depth = Some((
                Depth {
                    fun: Comparison::Always,
                    write: true,
                },
                Stencil::default(),
            ));
        }
        self.init.out_blends.push((name, mask, blend));
        self
    }

    /// Adds a texture sampler to this `Effect`.
    pub fn with_texture(&mut self, name: &'a str) -> &mut Self {
        self.init.samplers.push(name);
        self.init.textures.push(name);
        self
    }

    /// Adds a vertex buffer to this `Effect`.
    pub fn with_raw_vertex_buffer(
        &mut self,
        attrs: Attributes<'a>,
        stride: ElemStride,
        rate: InstanceRate,
    ) -> &mut Self {
        self.init.vertex_bufs.push((attrs, stride, rate));
        self
    }

    /// TODO: Support render targets as inputs.
    pub fn build(&mut self) -> Result<Effect> {
        use gfx::traits::FactoryExt;
        use gfx::Factory;

        debug!("Building effect");
        debug!("Compiling shaders");
        let ref mut fac = self.factory;
        let prog = self.prog.compile(fac)?;
        debug!("Creating pipeline state");
        let pso = fac.create_pipeline_state(&prog, self.prim, self.rast, self.init.clone())?;

        let mut data = Data::default();

        debug!("Creating raw constant buffers");
        let const_bufs = self
            .init
            .const_bufs
            .iter()
            .enumerate()
            .zip(self.const_bufs.drain(..))
            .map(|((i, name), info)| {
                let cbuf = fac.create_buffer_raw(info)?;
                data.const_bufs.push(cbuf);
                Ok((name.to_string(), i))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        debug!("Set global uniforms");
        let globals = self
            .init
            .globals
            .iter()
            .enumerate()
            .map(|(i, name)| {
                // Insert placeholder value until updated by user.
                data.globals.push(UniformValue::F32Vector4([0.0; 4]));
                (name.to_string(), i)
            })
            .collect::<HashMap<_, _>>();

        debug!("Process Color/Depth/Blend outputs");
        data.out_colors.extend(
            self.out
                .color_bufs()
                .iter()
                .map(|cb| &cb.as_output)
                .cloned(),
        );
        data.out_blends.extend(
            self.out
                .color_bufs()
                .iter()
                .map(|cb| &cb.as_output)
                .cloned(),
        );
        data.out_depth = self
            .out
            .depth_buf()
            .map(|db| (db.as_output.clone(), (0, 0)));

        debug!("Finished building effect");
        Ok(Effect {
            pso,
            data,
            const_bufs,
            globals,
        })
    }
}
