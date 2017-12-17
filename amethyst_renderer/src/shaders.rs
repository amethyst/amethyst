use std::collections::hash_map::{Entry, HashMap};
use std::path::{Path, PathBuf};

use gfx_hal::{Backend, Device};
use gfx_hal::device::ShaderError;
use gfx_hal::pso::{EntryPoint, GraphicsShaderSet, Stage};

error_chain!{
    errors {
        CompilationFailed(msg: String) {
            description("Shader compilation failed")
            display("Shader compilation failed:\n{}", msg)
        }

        MissingEntryPoint(msg: String) {
            description("Missing entry point")
            display("Missing entry point:\n{}", msg)
        }

        InterfaceMismatch(msg: String) {
            description("Interface mismatch")
            display("Interface mismatch:\n{}", msg)
        }

        ShaderNotFound(name: String, stage: Stage) {
            description("Shader not found")
            display("Shader {}:{:?} not found", name, stage)
        }
    }
}

fn map_shader_error(err: ShaderError) -> Error {
    match err {
        ShaderError::CompilationFailed(msg) => ErrorKind::CompilationFailed(msg),
        ShaderError::MissingEntryPoint(msg) => ErrorKind::MissingEntryPoint(msg),
        ShaderError::InterfaceMismatch(msg) => ErrorKind::InterfaceMismatch(msg),
    }.into()
}

#[derive(Clone, Copy)]
pub struct GraphicsShaderNameSet<'a> {
    pub vertex: &'a str,
    pub hull: Option<&'a str>,
    pub domain: Option<&'a str>,
    pub geometry: Option<&'a str>,
    pub fragment: Option<&'a str>,
}

impl<'a> GraphicsShaderNameSet<'a> {
    pub fn new(name: &'a str, hull: bool, domain: bool, geometry: bool, fragment: bool) -> Self {
        GraphicsShaderNameSet {
            vertex: name,
            hull: if hull { Some(name) } else { None },
            domain: if domain { Some(name) } else { None },
            geometry: if geometry { Some(name) } else { None },
            fragment: if fragment { Some(name) } else { None },
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug, Default(bound = ""))]
pub struct ShaderManager<B: Backend> {
    shaders_dir: PathBuf,
    shaders: HashMap<PathBuf, B::ShaderModule>,
}

impl<B> ShaderManager<B>
where
    B: ShaderLoader,
{
    pub fn new() -> Self {
        ShaderManager::default()
    }

    // TODO: Move to the config
    pub fn set_shaders_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.shaders_dir = path.as_ref().into();
    }

    pub fn load_shader_set<'a>(
        &'a mut self,
        names: GraphicsShaderNameSet,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>> {
        let GraphicsShaderNameSet {
            vertex,
            hull,
            domain,
            geometry,
            fragment,
        } = names;

        self.load_shader(vertex, Stage::Vertex, device)?;
        if let Some(hull) = hull {
            self.load_shader(hull, Stage::Hull, device)?;
        }
        if let Some(domain) = domain {
            self.load_shader(domain, Stage::Domain, device)?;
        }
        if let Some(geometry) = geometry {
            self.load_shader(geometry, Stage::Geometry, device)?;
        }
        if let Some(fragment) = fragment {
            self.load_shader(fragment, Stage::Fragment, device)?;
        }

        self.get_shader_set(names)
    }

    pub fn load_shader<'a>(
        &'a mut self,
        name: &str,
        stage: Stage,
        device: &B::Device,
    ) -> Result<EntryPoint<'a, B>> {
        let path = B::get_shader_path(&self.shaders_dir, name, stage);
        let module = match self.shaders.entry(path) {
            Entry::Occupied(occupied) => occupied.into_mut(),
            Entry::Vacant(vacant) => {
                let module = B::create_shader_module(device, &self.shaders_dir, name, stage)?;
                vacant.insert(module)
            }
        };
        Ok(EntryPoint {
            entry: B::get_shader_entry(stage),
            module,
            specialization: &[],
        })
    }

    pub fn get_shader_set<'a>(
        &'a self,
        names: GraphicsShaderNameSet,
    ) -> Result<GraphicsShaderSet<'a, B>> {
        let GraphicsShaderNameSet {
            vertex,
            hull,
            domain,
            geometry,
            fragment,
        } = names;

        let set = GraphicsShaderSet {
            vertex: self.get_shader(vertex, Stage::Vertex)?,
            hull: hull.map(|hull| self.get_shader(hull, Stage::Hull).map(Some))
                .unwrap_or(Ok(None))?,
            domain: domain
                .map(|domain| self.get_shader(domain, Stage::Domain).map(Some))
                .unwrap_or(Ok(None))?,
            geometry: geometry
                .map(|geometry| {
                    self.get_shader(geometry, Stage::Geometry).map(Some)
                })
                .unwrap_or(Ok(None))?,
            fragment: fragment
                .map(|fragment| {
                    self.get_shader(fragment, Stage::Fragment).map(Some)
                })
                .unwrap_or(Ok(None))?,
        };
        Ok(set)
    }

    pub fn get_shader<'a>(&'a self, name: &str, stage: Stage) -> Result<EntryPoint<'a, B>> {
        let path = B::get_shader_path(&self.shaders_dir, name, stage);
        self.shaders
            .get(&path)
            .map(|module| {
                EntryPoint {
                    entry: B::get_shader_entry(stage),
                    module,
                    specialization: &[],
                }
            })
            .ok_or_else(|| ErrorKind::ShaderNotFound(name.into(), stage).into())
    }
}

impl<B> ShaderManager<B>
where
    B: Backend,
{
    pub fn unload(&mut self, device: &B::Device) {
        for (_, shader) in self.shaders.drain() {
            device.destroy_shader_module(shader);
        }
    }
}

pub trait ShaderLoader: Backend {
    fn get_shader_path(prefix: &PathBuf, name: &str, stage: Stage) -> PathBuf;
    fn get_shader_entry(stage: Stage) -> &'static str;
    fn create_shader_module(
        device: &Self::Device,
        prefix: &PathBuf,
        name: &str,
        stage: Stage,
    ) -> Result<Self::ShaderModule>;
}

#[cfg(feature = "spirv")]
impl<B> ShaderLoader for B
where
    B: Backend,
{
    fn get_shader_path(prefix: &PathBuf, name: &str, stage: Stage) -> PathBuf {
        let mut path = prefix.clone();
        path.push(name);
        match stage {
            Stage::Vertex => path.push("vert"),
            Stage::Fragment => path.push("frag"),
            _ => unimplemented!(),
        }
        path.set_extension("spv");
        path
    }

    fn get_shader_entry(_stage: Stage) -> &'static str {
        "main"
    }

    fn create_shader_module(
        device: &B::Device,
        prefix: &PathBuf,
        name: &str,
        stage: Stage,
    ) -> Result<B::ShaderModule> {
        let path = Self::get_shader_path(prefix, name, stage);
        println!("Load shader {:?}", path);
        let code = load_shader_bytes(&path)?;
        let module = device
            .create_shader_module(&code)
            .map_err(map_shader_error)?;
        Ok(module)
    }
}

#[cfg(all(feature = "gfx-backend-metal", not(feature = "spirv")))]
impl ShaderLoader for ::metal::Backend {
    fn get_shader_path(prefix: &PathBuf, name: &str, _stage: Stage) -> PathBuf {
        let mut path = prefix.clone();
        path.push(name);
        path.set_extension("metal");
        path
    }

    fn get_shader_entry(stage: Stage) -> &'static str {
        match stage {
            Stage::Vertex => "vs_main",
            Stage::Fragment => "ps_main",
            _ => unimplemented!(),
        }
    }

    fn create_shader_module(
        device: &<::metal::Backend as Backend>::Device,
        prefix: &PathBuf,
        name: &str,
        stage: Stage,
    ) -> Result<<::metal::Backend as Backend>::ShaderModule> {
        let path = Self::get_shader_path(prefix, name, stage);
        println!("Load shader {:?}", path);
        let code = load_shader_code(&path)?;
        let version = ::metal::LanguageVersion::new(1, 2);
        let module = device
            .create_shader_library_from_source(&code, version)
            .map_err(map_shader_error)?;
        Ok(module)
    }
}


#[cfg(all(feature = "gfx-backend-vulkan", not(feature = "spirv")))]
impl ShaderLoader for ::vulkan::Backend {
    fn get_shader_path(prefix: &PathBuf, name: &str, stage: Stage) -> PathBuf {
        let mut path = prefix.clone();
        path.push(name);
        match stage {
            Stage::Vertex => path.set_extension("vert"),
            Stage::Fragment => path.set_extension("frag"),
            _ => unimplemented!(),
        };
        path
    }

    fn get_shader_entry(stage: Stage) -> &'static str {
        "main"
    }

    fn create_shader_module(
        device: &<::vulkan::Backend as Backend>::Device,
        prefix: &PathBuf,
        name: &str,
        stage: Stage,
    ) -> Result<<::vulkan::Backend as Backend>::ShaderModule> {
        let path = Self::get_shader_path(prefix, name, stage);
        println!("Load shader {:?}", path);
        let code = load_shader_code(&path)?;
        let module = device
            .create_shader_module_from_glsl(&code, stage)
            .map_err(map_shader_error)?;
        Ok(module)
    }
}

#[cfg(not(feature = "spirv"))]
fn load_shader_code(path: &Path) -> Result<String> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(path).chain_err(|| format!("Failed to open shader file {:?}", path))?;
    let mut code = String::new();
    file.read_to_string(&mut code)
        .chain_err(|| format!("Failed to read shader file {:?}", path))?;
    Ok(code)
}


#[cfg(feature = "spirv")]
fn load_shader_bytes(path: &Path) -> Result<Vec<u8>> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(path).chain_err(|| format!("Failed to open shader file {:?}", path))?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)
        .chain_err(|| format!("Failed to read shader file {:?}", path))?;
    Ok(code)
}
