//! Proposed API usage for when the rendering engine is complete.

extern crate amethyst_renderer;
use amethyst_renderer as r;

fn main() {
    // Right now, we have...

    let pipeline = r::Pipeline::build("forward")
                               .new_stage("geometry")
                               .new_stage("lighting")
                               .new_stage("postproc")
                               .done();

    let mut renderer = r::Renderer::new(pipeline);

    for _ in 0..5 {
        let frame = r::Frame::new();
        renderer.draw(frame);
    }

    // I'm looking for something more like this...

    // let mut renderer = r::Renderer::new_forward().expect("Whoops!");
    //
    // let mut res = r::Resources {...};
    // let mut handles = renderer.push_resources(res);
    //
    // loop {
    //     if user_wins_level() {
    //         renderer.pop_resources();
    //
    //         res = get_next_level_resources();
    //         handles = renderer.push_resources(res);
    //     }
    //
    //     let frame = r::Frame {...};
    //     renderer.draw(frame);
    // }

    // Alternatively, you can build everything yourself if you want to.

    // let targets = r::RenderTargets::build()
    //                                .new_target("output", r::Format::RGBA16F)
    //                                .with_color_buffers(1)
    //                                .with_depth_buffer()
    //                                .done();
    //
    // let pipe = r::Pipeline::build("forward")
    //                        .new_stage("geometry")
    //                            .use_target("output")
    //                            .clear_target(r::BufClear::All, [0.0; 4])
    //                            .draw_objects()
    //                        .new_stage("lighting")
    //                            .use_target("output")
    //                            .light_loop_forward(...)
    //                        .new_stage("postproc")
    //                            .use_target("output")
    //                            .apply_effect(...)
    //                        .new_stage("display")
    //                            .fullscreen_quad("output")
    //                        .done();
    //
    // let mut renderer = r::Renderer::new(targets, pipe).expect("Whoops!");
    //
    // <Your drawing loop here>
}
