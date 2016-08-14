use context::ContextConfig;
use processors::RendererConfig;

use config::Element;
use std::path::Path;

config!(
    struct Config {
        pub context_config: ContextConfig = ContextConfig::default(),
        pub renderer_config: RendererConfig = RendererConfig::default(),
    }
);
