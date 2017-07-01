//! Helper builder for pipeline state objects.

#![allow(missing_docs)]

use self::pso::{Data, Init, Meta};

use error::{Error, Result};
use fnv::FnvHashMap as HashMap;
use gfx::{Primitive, ShaderSet};
use gfx::preset::depth::{LESS_EQUAL_TEST, LESS_EQUAL_WRITE};
use gfx::shade::{ProgramError, ToUniform};
use gfx::state::{Depth, Rasterizer};
use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use pipe::{Target, Targets};
use types::{Factory, PipelineState, Resources, Sampler};

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
enum ProgramSource {
    Simple(&'static [u8], &'static [u8]),
    Geometry(&'static [u8], &'static [u8], &'static [u8]),
    Tessellated(&'static [u8], &'static [u8], &'static [u8], &'static [u8]),
}

impl ProgramSource {
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
    pso: PipelineState<Meta>,
    #[derivative(Hash = "ignore")]
    pso_data: Data,
    #[derivative(Hash = "ignore")]
    samplers: HashMap<String, Sampler>,
}

impl Effect {
    pub fn new_simple_prog<S>(vs: S, ps: S) -> EffectBuilder
        where S: Into<&'static [u8]>
    {
        EffectBuilder::new_simple_prog(vs, ps)
    }

    pub fn new_geom_prog<S>(vs: S, gs: S, ps: S) -> EffectBuilder
        where S: Into<&'static [u8]>
    {
        EffectBuilder::new_geom_prog(vs, gs, ps)
    }

    pub fn new_tess_prog<S>(vs: S, hs: S, ds: S, ps: S) -> EffectBuilder
        where S: Into<&'static [u8]>
    {
        EffectBuilder::new_tess_prog(vs, hs, ds, ps)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectBuilder {
    init: Init<'static>,
    out_depth: Depth,
    prim: Primitive,
    prog: ProgramSource,
    rast: Rasterizer,
    samplers: HashMap<String, SamplerInfo>,
}

impl Default for EffectBuilder {
    fn default() -> EffectBuilder {
        use gfx::Primitive;
        use gfx::state::Rasterizer;

        EffectBuilder {
            init: Init::default(),
            out_depth: LESS_EQUAL_WRITE,
            prim: Primitive::TriangleList,
            rast: Rasterizer::new_fill().with_cull_back(),
            prog: ProgramSource::Simple("".as_bytes(), "".as_bytes()),
            samplers: HashMap::default(),
        }
    }
}

impl EffectBuilder {
    pub fn new_simple_prog<S>(vs: S, ps: S) -> EffectBuilder
        where S: Into<&'static [u8]>
    {
        let (vs, ps) = (vs.into(), ps.into());
        EffectBuilder {
            prog: ProgramSource::Simple(vs, ps),
            .. Default::default()
        }
    }

    pub fn new_geom_prog<S>(vs: S, gs: S, ps: S) -> EffectBuilder
        where S: Into<&'static [u8]>
    {
        let (vs, gs, ps) = (vs.into(), gs.into(), ps.into());
        EffectBuilder {
            prog: ProgramSource::Geometry(vs, gs, ps),
            .. Default::default()
        }
    }

    pub fn new_tess_prog<S>(vs: S, hs: S, ds: S, ps: S) -> EffectBuilder
        where S: Into<&'static [u8]>
    {
        let (vs, hs, ds, ps) = (vs.into(), hs.into(), ds.into(), ps.into());
        EffectBuilder {
            prog: ProgramSource::Tessellated(vs, hs, ds, ps),
            .. Default::default()
        }
    }

    /// Adds a global constant to this `Effect`.
    pub fn with_global<N: Into<&'static str>>(mut self, name: N) -> Self {
        self.init.globals.push(name.into());
        self
    }

    /// Sets the output target of the PSO.
    ///
    /// If the target contains a depth buffer, its mode will be set by `depth`.
    pub fn with_output(mut self, depth: DepthMode) -> Self {
        self.out_depth = match depth {
            DepthMode::LessEqualTest => LESS_EQUAL_TEST,
            DepthMode::LessEqualWrite => LESS_EQUAL_WRITE,
        };
        self
    }

    /// Requests a new texture sampler be created for this `Effect`.
    pub fn with_sampler<N>(mut self, name: N, f: FilterMethod, w: WrapMode) -> Self
        where N: Into<&'static str>
    {
        let val = name.into();
        self.samplers.insert(val.to_string(), SamplerInfo::new(f, w));
        self.init.samplers.push(val);
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
            .map(|(name, info)| (name.clone(), fac.create_sampler(*info)))
            .collect::<HashMap<_, _>>();

        Ok(Effect {
            pso: pso,
            pso_data: Data::default(),
            samplers: samplers,
        })
    }
}
