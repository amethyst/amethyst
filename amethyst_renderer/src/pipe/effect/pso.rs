use gfx::{handle, pso};
use gfx::pso::{DataBind, DataLink, Descriptor, PipelineData, PipelineInit, InitError};
use gfx::pso::buffer::{RawConstantBuffer, RawGlobal, RawVertexBuffer};
use gfx::pso::resource::{RawShaderResource, Sampler};
use gfx::pso::target;
use gfx::shade::core::ProgramInfo;
use types::{ColorFormat, DepthFormat, Resources};

type AccessInfo = pso::AccessInfo<Resources>;
type DepthStencilTarget = target::DepthStencilTarget<DepthFormat>;
type Manager = handle::Manager<Resources>;
type RenderTarget = target::RenderTarget<ColorFormat>;
type RawDataSet = pso::RawDataSet<Resources>;
type InitResult<'r, M> = Result<M, InitError<&'r str>>;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Meta {
    const_bufs: Vec<RawConstantBuffer>,
    globals: Vec<RawGlobal>,
    out_colors: Vec<RenderTarget>,
    out_depth: Option<DepthStencilTarget>,
    samplers: Vec<Sampler>,
    textures: Vec<RawShaderResource>,
    vertex_bufs: Vec<RawVertexBuffer>,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Init<'d> {
    pub const_bufs: Vec<<RawConstantBuffer as DataLink<'d>>::Init>,
    pub globals: Vec<<RawGlobal as DataLink<'d>>::Init>,
    pub out_colors: Vec<<RenderTarget as DataLink<'d>>::Init>,
    pub out_depth: Option<<DepthStencilTarget as DataLink<'d>>::Init>,
    pub samplers: Vec<<Sampler as DataLink<'d>>::Init>,
    pub textures: Vec<<RawShaderResource as DataLink<'d>>::Init>,
    pub vertex_bufs: Vec<<RawVertexBuffer as DataLink<'d>>::Init>,
}

impl<'d> PipelineInit for Init<'d> {
    type Meta = Meta;

    fn link_to<'r>(&self, desc: &mut Descriptor, info: &'r ProgramInfo) -> InitResult<'r, Meta> {
        let mut meta = Meta::default();

        for (info, cbuf) in info.constant_buffers.iter().zip(&self.const_bufs) {
            let mut meta_cbuf = <RawConstantBuffer as DataLink<'d>>::new();
            if let Some(res) = meta_cbuf.link_constant_buffer(info, cbuf) {
                let d = res.map_err(|e| InitError::ConstantBuffer(info.name.as_str(), Some(e)))?;
                meta.const_bufs.push(meta_cbuf);
                desc.constant_buffers[info.slot as usize] = Some(d);
            }
        }

        for (info, global) in info.globals.iter().zip(&self.globals) {
            let mut meta_global = <RawGlobal as DataLink<'d>>::new();
            if let Some(res) = meta_global.link_global_constant(info, global) {
                res.map_err(|e| InitError::GlobalConstant(info.name.as_str(), Some(e)))?;
                meta.globals.push(meta_global);
            }
        }

        for (info, color) in info.outputs.iter().zip(&self.out_colors) {
            let mut meta_color = <RenderTarget as DataLink<'d>>::new();
            if let Some(res) = meta_color.link_output(info, color) {
                let d = res.map_err(|e| InitError::PixelExport(info.name.as_str(), Some(e)))?;
                meta.out_colors.push(meta_color);
                desc.color_targets[info.slot as usize] = Some(d);
            }
        }

        if let Some(depth) = self.out_depth {
            let mut meta_depth = <DepthStencilTarget as DataLink<'d>>::new();
            if let Some(d) = meta_depth.link_depth_stencil(&depth) {
                desc.scissor = meta_depth.link_scissor();
                meta.out_depth = Some(meta_depth);
                desc.depth_stencil = Some(d);
            }
        }

        for (info, smp) in info.samplers.iter().zip(&self.samplers) {
            let mut meta_smp = <Sampler as DataLink<'d>>::new();
            if let Some(d) = meta_smp.link_sampler(info, smp) {
                meta.samplers.push(meta_smp);
                desc.samplers[info.slot as usize] = Some(d);
            }
        }

        for (info, tex) in info.textures.iter().zip(&self.textures) {
            let mut meta_tex = <RawShaderResource as DataLink<'d>>::new();
            if let Some(res) = meta_tex.link_resource_view(info, tex) {
                let d = res.map_err(|_| InitError::ResourceView(info.name.as_str(), Some(())))?;
                meta.textures.push(meta_tex);
                desc.resource_views[info.slot as usize] = Some(d);
            }
        }

        for (i, vbuf) in self.vertex_bufs.iter().enumerate() {
            let mut meta_vbuf = <RawVertexBuffer as DataLink<'d>>::new();
            if let Some(d) = meta_vbuf.link_vertex_buffer(i as u8, vbuf) {
                for attr in info.vertex_attributes.iter() {
                    if let Some(res) = meta_vbuf.link_input(attr, vbuf) {
                        let d = res.map_err(|e| InitError::VertexImport(attr.name.as_str(), Some(e)))?;
                        desc.attributes[attr.slot as usize] = Some(d);
                    }
                }

                meta.vertex_bufs.push(meta_vbuf);
                desc.vertex_buffers[i] = Some(d);
            }
        }

        Ok(meta)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Data {
    pub const_bufs: Vec<<RawConstantBuffer as DataBind<Resources>>::Data>,
    pub globals: Vec<<RawGlobal as DataBind<Resources>>::Data>,
    pub out_colors: Vec<<RenderTarget as DataBind<Resources>>::Data>,
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
