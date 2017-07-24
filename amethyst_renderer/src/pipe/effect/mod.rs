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
use gfx::state::{Comparison, Depth, Rasterizer, Stencil};
use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use pipe::{Target, Targets};
use types::{Factory, PipelineState, RawBuffer, Resources, Sampler};
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum ProgramSource<'a> {
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
#[derivative(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Effect {
    pub pso: PipelineState<Meta>,
    #[derivative(Hash = "ignore")]
    pub pso_data: Data,
    #[derivative(Hash = "ignore")]
    pub samplers: HashMap<String, Sampler>,
    #[derivative(Hash = "ignore")]
    pub const_bufs: HashMap<String, RawBuffer>,
}

impl Effect {
    pub fn new_simple_prog<'a, S>(vs: S, ps: S) -> EffectBuilder<'a>
        where S: Into<&'a [u8]>
    {
        EffectBuilder::new_simple_prog(vs, ps)
    }

    pub fn new_geom_prog<'a, S>(vs: S, gs: S, ps: S) -> EffectBuilder<'a>
        where S: Into<&'a [u8]>
    {
        EffectBuilder::new_geom_prog(vs, gs, ps)
    }

    pub fn new_tess_prog<'a, S>(vs: S, hs: S, ds: S, ps: S) -> EffectBuilder<'a>
        where S: Into<&'a [u8]>
    {
        EffectBuilder::new_tess_prog(vs, hs, ds, ps)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectBuilder<'a> {
    init: Init<'a>,
    prim: Primitive,
    prog: ProgramSource<'a>,
    rast: Rasterizer,
    samplers: HashMap<SamplerInfo, &'a [&'a str]>,
    const_bufs: HashMap<&'a str, BufferInfo>,
}

impl<'a> Default for EffectBuilder<'a> {
    fn default() -> Self {
        use gfx::Primitive;
        use gfx::state::Rasterizer;

        EffectBuilder {
            init: Init::default(),
            prim: Primitive::TriangleList,
            rast: Rasterizer::new_fill().with_cull_back(),
            prog: ProgramSource::Simple("".as_bytes(), "".as_bytes()),
            samplers: HashMap::default(),
            const_bufs: HashMap::default(),
        }
    }
}

impl<'a> EffectBuilder<'a> {
    pub fn new_simple_prog<S>(vs: S, ps: S) -> Self
        where S: Into<&'a [u8]>
    {
        let (vs, ps) = (vs.into(), ps.into());
        EffectBuilder {
            prog: ProgramSource::Simple(vs, ps),
            .. Default::default()
        }
    }

    pub fn new_geom_prog<S>(vs: S, gs: S, ps: S) -> Self
        where S: Into<&'a [u8]>
    {
        let (vs, gs, ps) = (vs.into(), gs.into(), ps.into());
        EffectBuilder {
            prog: ProgramSource::Geometry(vs, gs, ps),
            .. Default::default()
        }
    }

    pub fn new_tess_prog<S>(vs: S, hs: S, ds: S, ps: S) -> Self
        where S: Into<&'a [u8]>
    {
        let (vs, hs, ds, ps) = (vs.into(), hs.into(), ds.into(), ps.into());
        EffectBuilder {
            prog: ProgramSource::Tessellated(vs, hs, ds, ps),
            .. Default::default()
        }
    }

    /// Adds a global constant to this `Effect`.
    pub fn with_raw_global(mut self, name: &'a str) -> Self {
        self.init.globals.push(name);
        self
    }

    /// Adds a raw uniform constant to this `Effect`.
    /// Requests a new constant buffer to be created
    pub fn with_raw_constant_buffer(mut self, name: &'a str, size: usize, num: usize) -> Self {
        self.const_bufs.insert(name, BufferInfo {
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

    /// Requests a new texture sampler be created for this `Effect`.
    pub fn with_sampler(mut self, names: &'a [&'a str], f: FilterMethod, w: WrapMode) -> Self {
        self.samplers.insert(SamplerInfo::new(f, w), names);
        self.init.samplers.extend(names);
        self
    }

    /// Adds a texture to this `Effect`
    pub fn with_texture(mut self, name: &'a str) -> Self {
        self.init.textures.push(name);
        self
    }

    /// Adds a vertex buffer to this `Effect`
    pub fn with_raw_vertex_buffer(mut self, attributes: &'a [(&'a str, Attribute)], stride: ElemStride, rate: InstanceRate) -> Self {
        self.init.vertex_bufs.push((attributes, stride, rate));
        self
    }

    pub(crate) fn finish(self, fac: &mut Factory, out: &Target) -> Result<Effect> {
        use gfx::Factory;
        use gfx::traits::FactoryExt;

        let prog = self.prog.compile(fac)?;
        let pso = fac.create_pipeline_state(&prog, self.prim, self.rast, self.init)?;

        let samplers = self.samplers
            .clone()
            .iter()
            .flat_map(|(info, names)| {
                let sampler = fac.create_sampler(*info);
                names.iter().map(move |name| ((*name).into(), sampler.clone()))
            })
            .collect::<HashMap<_, _>>();

        let const_bufs = self.const_bufs
            .clone()
            .iter()
            .map(|(name, info)| Ok(((*name).into(), fac.create_buffer_raw(*info)?)))
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(Effect {
            pso: pso,
            pso_data: Data::default(),
            samplers: samplers,
            const_bufs: const_bufs,
        })
    }
}
