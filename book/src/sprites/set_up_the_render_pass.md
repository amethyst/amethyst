# Set Up The Render Pass

Amethyst supports drawing sprites using the `DrawFlat2DDesc` render descriptor.
To enable this you have to do the following:

1. Build a `Subpass` in your `GraphCreator`.
2. Add the `DrawFlat2DDesc` descriptor to the pass.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::{
#     assets::Processor,
#     ecs::{ReadExpect, Resources, SystemData},
#     prelude::*,
#     renderer::{
#         pass::DrawFlat2DDesc, types::DefaultBackend, Factory, Format, GraphBuilder, GraphCreator,
#         Kind, RenderGroupDesc, RenderingSystem, SpriteSheet, SubpassBuilder,
#     },
#     ui::DrawUiDesc,
#     utils::application_root_dir,
#     window::{ScreenDimensions, Window, WindowBundle},
# };
# #[derive(Default)]
# struct ExampleGraph {
#     dimensions: Option<ScreenDimensions>,
#     dirty: bool,
# }
# 
# impl GraphCreator<DefaultBackend> for ExampleGraph {
#     fn rebuild(&mut self, world: &World) -> bool {
#         let new_dimensions = res.try_fetch::<ScreenDimensions>();
#         use std::ops::Deref;
#         if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
#             self.dirty = true;
#             self.dimensions = new_dimensions.map(|d| d.clone());
#             return false;
#         }
#         return self.dirty;
#     }
# 
#     fn builder(
#         &mut self,
#         factory: &mut Factory<DefaultBackend>,
#         world: &World,
#     ) -> GraphBuilder<DefaultBackend, Resources> {
#         use amethyst::renderer::rendy::{
#             graph::present::PresentNode,
#             hal::command::{ClearDepthStencil, ClearValue},
#         };
# 
#         self.dirty = false;
# 
#         // Retrieve a reference to the target window, which is created by the WindowBundle
#         let window = <ReadExpect<'_, Window>>::fetch(res);
#         let dimensions = self.dimensions.as_ref().unwrap();
#         let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);
# 
#         // Create a new drawing surface in our window
#         let surface = factory.create_surface(&window);
#         let surface_format = factory.get_surface_format(&surface);
# 
#         // Begin building our RenderGraph
#         let mut graph_builder = GraphBuilder::new();
#         let color = graph_builder.create_image(
#             window_kind,
#             1,
#             surface_format,
#             // clear screen to black
#             Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
#         );
# 
#         let depth = graph_builder.create_image(
#             window_kind,
#             1,
#             Format::D32Sfloat,
#             Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
#         );
#
// Create our first `Subpass`, which contains the DrawFlat2D render group.
// We pass the subpass builder a description of our groups for construction
let pass = graph_builder.add_node(
    SubpassBuilder::new()
        .with_group(DrawFlat2DDesc::default().builder()) // Draws sprites
        .with_color(color)
        .with_depth_stencil(depth)
        .into_pass(),
);
#
#         // Finally, add the pass to the graph
#         let _present = graph_builder
#             .add_node(PresentNode::builder(factory, surface, color).with_dependency(pass));
# 
#         graph_builder
#     }
# }
```
