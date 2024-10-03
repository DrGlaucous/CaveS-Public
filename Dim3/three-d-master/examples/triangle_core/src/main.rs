use three_d::core::{
    degrees, radians, vec3, ClearState, Context, Mat4, Program, RenderStates, Srgba, VertexBuffer,
};
use three_d::window::{FrameOutput, Window, WindowSettings};
use three_d::{CoreError, HasContext, Viewport};
use three_d_asset::Camera;

use three_d::context::Context as GContext;
use three_d::context::COLOR_BUFFER_BIT;

use std::time::{Duration, Instant};


pub fn main() {

    // // Create a window (a canvas on web)
    // let window = Window::new(WindowSettings {
    //     title: "Core Triangle!".to_string(),
    //     #[cfg(not(target_arch = "wasm32"))]
    //     max_size: Some((1280, 720)),
    //     ..Default::default()
    // })
    // .unwrap();

    // let ctt = window.gl();
    // ctt.


    //create a window with sdl
    #[cfg(feature = "sdl2")]
    let (gl, shader_version, window, mut events_loop, _context) = {
        unsafe{
            let sdl = sdl2::init().unwrap();
            let video = sdl.video().unwrap();
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 0);
            let window = video
                .window("Hello triangle!", 1024, 769)
                .opengl()
                .resizable()
                .build()
                .unwrap();
            let gl_context = window.gl_create_context().unwrap();
            
            let gl =
                GContext::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
            let event_loop = sdl.event_pump().unwrap();
            (gl, "#version 130", window, event_loop, gl_context)
        }
    };


    // Get the graphics context from the window
    let context: Context = Context::from_gl_context(gl.into()).unwrap();

    // Define triangle vertices and colors
    let positions = VertexBuffer::new_with_data(
        &context,
        &[
            vec3(0.5, -0.5, 0.0),  // bottom right
            vec3(-0.5, -0.5, 0.0), // bottom left
            vec3(0.0, 0.5, 0.0),   // top
        ],
    );
    let colors = VertexBuffer::new_with_data(
        &context,
        &[
            Srgba::RED.to_linear_srgb(),   // bottom right
            Srgba::GREEN.to_linear_srgb(), // bottom left
            Srgba::BLUE.to_linear_srgb(),  // top
        ],
    );

    let program = Program::from_source(
        &context,
        include_str!("triangle.vert"),
        include_str!("triangle.frag"),
    )
    .unwrap();


    let window_size = window.size();

    let vp = Viewport {
        x: 0,
        y: 0,
        width: window_size.0,
        height: window_size.1,
    };

    let mut camera = Camera::new_perspective(
        vp,
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        10.0,
    );

    #[cfg(feature = "sdl2")]
    {
        let mut running = true;
        let mut time = 0.0;
        window.gl_swap_window();
        while running {
            {
                for event in events_loop.poll_iter() {
                    match event {
                        sdl2::event::Event::Quit { .. } => running = false,
                        _ => {}
                    }
                }
            }

            //gl.clear(glow::COLOR_BUFFER_BIT);
            //gl.draw_arrays(glow::TRIANGLES, 0, 3);
            unsafe {
                //context.clear(COLOR_BUFFER_BIT);
                //context.bind_buffer(target, buffer);
                //context.set_blend(blend);
            }
            

            time = 100.0;//1.0; //frame_input.accumulated_time as f32;
            program.use_uniform("model", Mat4::from_angle_y(radians(time * 0.005)));
            program.use_uniform("viewProjection", camera.projection() * camera.view());
            program.use_vertex_attribute("position", &positions);
            program.use_vertex_attribute("color", &colors);
            program.draw_arrays(
                RenderStates::default(),
                vp, //frame_input.viewport,
                positions.vertex_count(),
            );

            
            window.gl_swap_window();

            std::thread::sleep(Duration::from_millis(1000));


            

            

            if !running {
                //gl.delete_program(program);
                //gl.delete_vertex_array(vertex_array);
            }
        }
    }



    //old, integrated stuff:
    /*
        let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        10.0,
    );


    window.render_loop(move |frame_input| {
        camera.set_viewport(frame_input.viewport);

        frame_input
            .screen()
            // Clear the color and depth of the screen render target
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .write::<CoreError>(|| {
                let time = frame_input.accumulated_time as f32;
                program.use_uniform("model", Mat4::from_angle_y(radians(time * 0.005)));
                program.use_uniform("viewProjection", camera.projection() * camera.view());
                program.use_vertex_attribute("position", &positions);
                program.use_vertex_attribute("color", &colors);
                program.draw_arrays(
                    RenderStates::default(),
                    frame_input.viewport,
                    positions.vertex_count(),
                );
                Ok(())
            })
            .unwrap();

        FrameOutput::default()
    });

    */










}
