//! Helper builder for pipeline state objects.

#![allow(missing_docs)]

use error::Result;
use fnv::FnvHashMap as HashMap;
use gfx::preset::depth::{LESS_EQUAL_TEST, LESS_EQUAL_WRITE};
use gfx::pso::Descriptor;
use gfx::state::Depth;
use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use pipe::{Target, Targets};
use types::{Factory, Program, RawPipelineState, Sampler};

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
    pub fn compile(&self, fac: &mut Factory) -> Result<Program> {
        use gfx::{Factory, ShaderSet};
        use gfx::shade::ProgramError;
        use gfx::traits::FactoryExt;

        match *self {
            ProgramSource::Simple(ref vs, ref ps) => {
                let set = fac.create_shader_set(vs, ps)?;
                fac.create_program(&set)
                    .map_err(|e| ProgramError::Link(e).into())
            }
            ProgramSource::Geometry(ref vs, ref gs, ref ps) => {
                let v = fac.create_shader_vertex(vs)
                    .map_err(|e| ProgramError::Vertex(e))?;
                let g = fac.create_shader_geometry(gs)
                    .expect("Geometry shader creation failed");
                let p = fac.create_shader_pixel(ps)
                    .map_err(|e| ProgramError::Pixel(e))?;
                let set = ShaderSet::Geometry(v, g, p);
                fac.create_program(&set)
                    .map_err(|e| ProgramError::Link(e).into())
            }
            ProgramSource::Tessellated(ref vs, ref hs, ref ds, ref ps) => {
                let set = fac.create_shader_set_tessellation(vs, hs, ds, ps)?;
                fac.create_program(&set)
                    .map_err(|e| ProgramError::Link(e).into())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Effect {
    pso: RawPipelineState,
    samplers: HashMap<String, Sampler>,
}

impl Effect {
    pub fn new() -> EffectBuilder {
        EffectBuilder::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectBuilder {
    desc: Descriptor,
    out_depth: Depth,
    prog: ProgramSource,
    samplers: HashMap<String, SamplerInfo>,
}

impl EffectBuilder {
    /// Creates a new `EffectBuilder`.
    pub fn new() -> EffectBuilder {
        use gfx::Primitive;
        use gfx::state::Rasterizer;

        let prim = Primitive::TriangleList;
        let rast = Rasterizer::new_fill().with_cull_back();

        EffectBuilder {
            desc: Descriptor::new(prim, rast),
            out_depth: LESS_EQUAL_WRITE,
            prog: ProgramSource::Simple("".as_bytes(), "".as_bytes()),
            samplers: HashMap::default(),
        }
    }

    /// Requests a new texture sampler be created for this `Pass`.
    pub fn with_sampler<N>(mut self, name: N, f: FilterMethod, w: WrapMode) -> Self
        where N: Into<String>
    {
        use gfx::shade::Usage;
        self.samplers.insert(name.into(), SamplerInfo::new(f, w));
        self.desc.samplers[self.samplers.len()] = Some(Usage::empty());
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

    pub fn with_simple_prog<S>(mut self, vs: S, ps: S) -> Self
        where S: Into<&'static [u8]>
    {
        let (vs, ps) = (vs.into(), ps.into());
        self.prog = ProgramSource::Simple(vs, ps);
        self
    }

    pub fn with_geom_prog<S>(mut self, vs: S, gs: S, ps: S) -> Self
        where S: Into<&'static [u8]>
    {
        let (vs, gs, ps) = (vs.into(), gs.into(), ps.into());
        self.prog = ProgramSource::Geometry(vs, gs, ps);
        self
    }

    pub fn with_tess_prog<S>(mut self, vs: S, hs: S, ds: S, ps: S) -> Self
        where S: Into<&'static [u8]>
    {
        let (vs, hs, ds, ps) = (vs.into(), hs.into(), ds.into(), ps.into());
        self.prog = ProgramSource::Tessellated(vs, hs, ds, ps);
        self
    }

    pub fn build(&mut self, fac: &mut Factory, out: &Target) -> Result<Effect> {
        use gfx::Factory;
        use gfx::format::Formatted;
        use gfx::state::MASK_ALL;
        use gfx_core::pso::{ColorInfo, DepthStencilInfo};
        use types::{ColorFormat, DepthFormat};

        for i in 0..out.color_bufs().len() {
            let fmt = ColorFormat::get_format();
            self.desc.color_targets[i] = Some((fmt,
                                               ColorInfo {
                                                   mask: MASK_ALL,
                                                   color: None,
                                                   alpha: None,
                                               }));
        }

        if out.depth_buf().is_some() {
            let fmt = DepthFormat::get_format();
            self.desc.depth_stencil = Some((fmt,
                                            DepthStencilInfo {
                                                depth: Some(self.out_depth.clone()),
                                                front: None,
                                                back: None,
                                            }));
        }

        let prog = self.prog.compile(fac)?;
        let pso = fac.create_pipeline_state_raw(&prog, &self.desc)?;

        let samplers = self.samplers
            .clone()
            .iter()
            .map(|(name, info)| (name.clone(), fac.create_sampler(*info)))
            .collect();

        Ok(Effect {
               pso: pso,
               samplers: samplers,
           })
    }
}
