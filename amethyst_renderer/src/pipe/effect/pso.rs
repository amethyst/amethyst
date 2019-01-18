use gfx::{
    handle,
    pso::{
        self,
        buffer::{RawConstantBuffer, RawGlobal, RawVertexBuffer},
        resource::{RawShaderResource, Sampler},
        target, DataBind, DataLink, Descriptor, InitError, PipelineData, PipelineInit,
    },
    shade::core::{BaseType, ContainerType, OutputVar, ProgramInfo},
};
use serde::Serialize;

use crate::types::{ColorFormat, DepthFormat, Resources};

type AccessInfo = pso::AccessInfo<Resources>;
type DepthStencilTarget = target::DepthStencilTarget<DepthFormat>;
type Manager = handle::Manager<Resources>;
type RenderTarget = target::RenderTarget<ColorFormat>;
type BlendTarget = target::BlendTarget<ColorFormat>;
type RawDataSet = pso::RawDataSet<Resources>;
type InitResult<'r, M> = Result<M, InitError<&'r str>>;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Meta {
    const_bufs: Vec<RawConstantBuffer>,
    globals: Vec<RawGlobal>,
    out_colors: Vec<RenderTarget>,
    out_blends: Vec<BlendTarget>,
    out_depth: Option<DepthStencilTarget>,
    samplers: Vec<Sampler>,
    textures: Vec<RawShaderResource>,
    vertex_bufs: Vec<RawVertexBuffer>,
}

/// FIXME: Cannot `Deserialize` for some reason.
#[derive(Clone, Debug, Default, Eq, Hash, Serialize, PartialEq)]
pub struct Init<'d> {
    pub const_bufs: Vec<<RawConstantBuffer as DataLink<'d>>::Init>,
    pub globals: Vec<<RawGlobal as DataLink<'d>>::Init>,
    pub out_colors: Vec<<RenderTarget as DataLink<'d>>::Init>,
    pub out_blends: Vec<<BlendTarget as DataLink<'d>>::Init>,
    pub out_depth: Option<<DepthStencilTarget as DataLink<'d>>::Init>,
    pub samplers: Vec<<Sampler as DataLink<'d>>::Init>,
    pub textures: Vec<<RawShaderResource as DataLink<'d>>::Init>,
    pub vertex_bufs: Vec<<RawVertexBuffer as DataLink<'d>>::Init>,
}

impl<'d> PipelineInit for Init<'d> {
    type Meta = Meta;

    fn link_to<'r>(&self, desc: &mut Descriptor, info: &'r ProgramInfo) -> InitResult<'r, Meta> {
        let mut meta = Meta::default();

        for cbuf in &self.const_bufs {
            let mut meta_cbuf = <RawConstantBuffer as DataLink<'d>>::new();
            for info in &info.constant_buffers {
                if let Some(res) = meta_cbuf.link_constant_buffer(info, cbuf) {
                    let d =
                        res.map_err(|e| InitError::ConstantBuffer(info.name.as_str(), Some(e)))?;
                    desc.constant_buffers[info.slot as usize] = Some(d);
                    break;
                }
            }
            meta.const_bufs.push(meta_cbuf);
        }

        for global in &self.globals {
            let mut meta_global = <RawGlobal as DataLink<'d>>::new();
            for info in &info.globals {
                if let Some(res) = meta_global.link_global_constant(info, global) {
                    res.map_err(|e| InitError::GlobalConstant(info.name.as_str(), Some(e)))?;
                    break;
                }
            }
            meta.globals.push(meta_global);
        }

        for color in &self.out_colors {
            let mut meta_color = <RenderTarget as DataLink<'d>>::new();
            for info in &info.outputs {
                if let Some(res) = meta_color.link_output(info, color) {
                    let d = res.map_err(|e| InitError::PixelExport(info.name.as_str(), Some(e)))?;
                    desc.color_targets[info.slot as usize] = Some(d);
                    break;
                }
            }
            meta.out_colors.push(meta_color);
        }

        for blend in &self.out_blends {
            let mut meta_blend = <BlendTarget as DataLink<'d>>::new();
            for info in &info.outputs {
                if let Some(res) = meta_blend.link_output(info, blend) {
                    let d = res.map_err(|e| InitError::PixelExport(info.name.as_str(), Some(e)))?;
                    desc.color_targets[info.slot as usize] = Some(d);
                    break;
                }
            }
            meta.out_blends.push(meta_blend);
        }

        if !info.knows_outputs {
            let mut info = OutputVar {
                name: String::new(),
                slot: 0,
                base_type: BaseType::F32,
                container: ContainerType::Vector(4),
            };

            for color in &self.out_colors {
                let mut meta_color = <RenderTarget as DataLink<'d>>::new();
                if let Some(res) = meta_color.link_output(&info, color) {
                    let d = res.map_err(|e| InitError::PixelExport("", Some(e)))?;
                    desc.color_targets[info.slot as usize] = Some(d);
                    info.slot += 1;
                }
                meta.out_colors.push(meta_color);
            }

            for blend in &self.out_blends {
                let mut meta_blend = <BlendTarget as DataLink<'d>>::new();
                if let Some(res) = meta_blend.link_output(&info, blend) {
                    let d = res.map_err(|e| InitError::PixelExport("", Some(e)))?;
                    desc.color_targets[info.slot as usize] = Some(d);
                    info.slot += 1;
                }
                meta.out_blends.push(meta_blend);
            }
        }

        if let Some(depth) = self.out_depth {
            let mut meta_depth = <DepthStencilTarget as DataLink<'d>>::new();
            if let Some(d) = meta_depth.link_depth_stencil(&depth) {
                desc.scissor = meta_depth.link_scissor();
                desc.depth_stencil = Some(d);
            }
            meta.out_depth = Some(meta_depth);
        }

        for smp in &self.samplers {
            let mut meta_smp = <Sampler as DataLink<'d>>::new();
            for info in &info.samplers {
                if let Some(d) = meta_smp.link_sampler(info, smp) {
                    desc.samplers[info.slot as usize] = Some(d);
                    break;
                }
            }
            meta.samplers.push(meta_smp);
        }

        for tex in &self.textures {
            let mut meta_tex = <RawShaderResource as DataLink<'d>>::new();
            for info in &info.textures {
                if let Some(res) = meta_tex.link_resource_view(info, tex) {
                    let d =
                        res.map_err(|_| InitError::ResourceView(info.name.as_str(), Some(())))?;
                    desc.resource_views[info.slot as usize] = Some(d);
                    break;
                }
            }
            meta.textures.push(meta_tex);
        }

        for (i, vbuf) in self.vertex_bufs.iter().enumerate() {
            let mut meta_vbuf = <RawVertexBuffer as DataLink<'d>>::new();
            if let Some(d) = meta_vbuf.link_vertex_buffer(i as u8, vbuf) {
                for attr in &info.vertex_attributes {
                    if let Some(res) = meta_vbuf.link_input(attr, vbuf) {
                        let d =
                            res.map_err(|e| InitError::VertexImport(attr.name.as_str(), Some(e)))?;
                        desc.attributes[attr.slot as usize] = Some(d);
                    }
                }
                desc.vertex_buffers[i] = Some(d);
            }
            meta.vertex_bufs.push(meta_vbuf);
        }

        Ok(meta)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Data {
    pub const_bufs: Vec<<RawConstantBuffer as DataBind<Resources>>::Data>,
    pub globals: Vec<<RawGlobal as DataBind<Resources>>::Data>,
    pub out_colors: Vec<<RenderTarget as DataBind<Resources>>::Data>,
    pub out_blends: Vec<<BlendTarget as DataBind<Resources>>::Data>,
    pub out_depth: Option<<DepthStencilTarget as DataBind<Resources>>::Data>,
    pub samplers: Vec<<Sampler as DataBind<Resources>>::Data>,
    pub textures: Vec<<RawShaderResource as DataBind<Resources>>::Data>,
    pub vertex_bufs: Vec<<RawVertexBuffer as DataBind<Resources>>::Data>,
}

impl PipelineData<Resources> for Data {
    type Meta = Meta;

    fn bake_to(&self, out: &mut RawDataSet, meta: &Meta, mgr: &mut Manager, acc: &mut AccessInfo) {
        let const_bufs = meta.const_bufs.iter().zip(&self.const_bufs);
        for (meta_cbuf, cbuf) in const_bufs {
            meta_cbuf.bind_to(out, &cbuf, mgr, acc);
        }

        let globals = meta.globals.iter().zip(&self.globals);
        for (meta_global, global) in globals {
            meta_global.bind_to(out, &global, mgr, acc);
        }

        let out_colors = meta.out_colors.iter().zip(&self.out_colors);
        for (meta_color, color) in out_colors {
            meta_color.bind_to(out, &color, mgr, acc);
        }

        let out_blends = meta.out_blends.iter().zip(&self.out_blends);
        for (meta_blend, blend) in out_blends {
            meta_blend.bind_to(out, &blend, mgr, acc);
        }

        let out_depth = (meta.out_depth.as_ref(), self.out_depth.as_ref());
        if let (Some(ref meta_depth), Some(ref depth)) = out_depth {
            meta_depth.bind_to(out, &depth, mgr, acc);
        }

        let samplers = meta.samplers.iter().zip(&self.samplers);
        for (meta_samp, samp) in samplers {
            meta_samp.bind_to(out, &samp, mgr, acc);
        }

        let textures = meta.textures.iter().zip(&self.textures);
        for (meta_tex, tex) in textures {
            meta_tex.bind_to(out, &tex, mgr, acc);
        }

        let vertex_bufs = meta.vertex_bufs.iter().zip(&self.vertex_bufs);
        for (meta_vbuf, vbuf) in vertex_bufs {
            meta_vbuf.bind_to(out, &vbuf, mgr, acc);
        }
    }
}
