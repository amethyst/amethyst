//! Graphical texture resource.

use ecs::{Component, VecStorage};
use renderer::prelude::{Material, MaterialBuilder, Pod};
use renderer::{Renderer, Result};
use super::unfinished::{ComponentBuilder, IntoUnfinished, Unfinished};

/// Wraps `Material` into component
pub struct MaterialComponent(pub Material);

impl Component for MaterialComponent {
    type Storage = VecStorage<Self>;
}

impl<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC> ComponentBuilder for MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>
    where DA: AsRef<[TA]>,
          TA: Pod,
          DE: AsRef<[TE]>,
          TE: Pod,
          DN: AsRef<[TN]>,
          TN: Pod,
          DM: AsRef<[TM]>,
          TM: Pod,
          DR: AsRef<[TR]>,
          TR: Pod,
          DO: AsRef<[TO]>,
          TO: Pod,
          DC: AsRef<[TC]>,
          TC: Pod,
{
    type Output = MaterialComponent;
    fn build(self: Box<Self>, renderer: &mut Renderer) -> Result<MaterialComponent> {
        renderer.create_material(*self).map(MaterialComponent)
    }
}

impl<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC> IntoUnfinished for MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>
    where DA: AsRef<[TA]> + Send + Sync + 'static,
          TA: Pod + Send + Sync + 'static,
          DE: AsRef<[TE]> + Send + Sync + 'static,
          TE: Pod + Send + Sync + 'static,
          DN: AsRef<[TN]> + Send + Sync + 'static,
          TN: Pod + Send + Sync + 'static,
          DM: AsRef<[TM]> + Send + Sync + 'static,
          TM: Pod + Send + Sync + 'static,
          DR: AsRef<[TR]> + Send + Sync + 'static,
          TR: Pod + Send + Sync + 'static,
          DO: AsRef<[TO]> + Send + Sync + 'static,
          TO: Pod + Send + Sync + 'static,
          DC: AsRef<[TC]> + Send + Sync + 'static,
          TC: Pod + Send + Sync + 'static,
{
    type Output = MaterialComponent;
    fn unfinished(self) -> Unfinished<MaterialComponent> {
        Unfinished::new(self)
    }
}