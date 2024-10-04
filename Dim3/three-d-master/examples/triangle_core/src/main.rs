//includes for "core" demo
use three_d::core::{
    degrees, radians, vec3, ClearState, Context, Mat4, Program, RenderStates, Srgba, VertexBuffer,
};
use three_d::window::{FrameOutput, Window, WindowSettings};
//use three_d_asset::Camera;
use three_d::{CoreError, HasContext, Viewport};
use three_d::context::Context as GContext;
use three_d::context;

use three_d::*;


pub fn main() {


    //high-level shapes
    {

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

        let window_size = window.size();

        let vp = Viewport {
            x: 0,
            y: 0,
            width: window_size.0,
            height: window_size.1,
        };

        let mut camera = Camera::new_perspective(
            vp, //window.viewport(),
            vec3(5.0, 2.0, 2.5),
            vec3(0.0, 0.0, -0.5),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        );
        let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);
    
        let mut sphere = Gm::new(
            Mesh::new(&context, &CpuMesh::sphere(16)),
            PhysicalMaterial::new_transparent(
                &context,
                &CpuMaterial {
                    albedo: Srgba {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 200,
                    },
                    ..Default::default()
                },
            ),
        );
        sphere.set_transformation(Mat4::from_translation(vec3(0.0, 1.3, 0.0)) * Mat4::from_scale(0.2));
        let mut cylinder = Gm::new(
            Mesh::new(&context, &CpuMesh::cylinder(16)),
            PhysicalMaterial::new_transparent(
                &context,
                &CpuMaterial {
                    albedo: Srgba {
                        r: 0,
                        g: 255,
                        b: 0,
                        a: 200,
                    },
                    ..Default::default()
                },
            ),
        );
        cylinder
            .set_transformation(Mat4::from_translation(vec3(1.3, 0.0, 0.0)) * Mat4::from_scale(0.2));
        let mut cube = Gm::new(
            Mesh::new(&context, &CpuMesh::cube()),
            PhysicalMaterial::new_transparent(
                &context,
                &CpuMaterial {
                    albedo: Srgba {
                        r: 0,
                        g: 0,
                        b: 255,
                        a: 100,
                    },
                    ..Default::default()
                },
            ),
        );
        cube.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 1.3)) * Mat4::from_scale(0.2));
        let axes = Axes::new(&context, 0.1, 2.0);
        let bounding_box_sphere = Gm::new(
            BoundingBox::new(&context, sphere.aabb()),
            ColorMaterial {
                color: Srgba::BLACK,
                ..Default::default()
            },
        );
        let bounding_box_cube = Gm::new(
            BoundingBox::new(&context, cube.aabb()),
            ColorMaterial {
                color: Srgba::BLACK,
                ..Default::default()
            },
        );
        let bounding_box_cylinder = Gm::new(
            BoundingBox::new(&context, cylinder.aabb()),
            ColorMaterial {
                color: Srgba::BLACK,
                ..Default::default()
            },
        );
    
        let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
        let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));



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


                //this is done for us by the scc
                // unsafe {
                //     context.clear_color(0.0, 0.0, 0.0, 1.0);
                //     context.clear(context::COLOR_BUFFER_BIT | context::DEPTH_BUFFER_BIT);
                //     //context.bind_buffer(target, buffer);
                //     //context.set_blend(blend);
                //     //context.bind_framebuffer(context::FRAMEBUFFER, Some(32));
                // }
                
                // Ensure the viewport matches the current window viewport which changes if the window is resized
                camera.set_viewport(vp);//(frame_input.viewport);

                //update with events (todo: make an SDL converter for these)
                //control.handle_events(&mut camera, &mut frame_input.events);


                let scc = RenderTarget::screen(&context, vp.width, vp.height);

                //we may be able to make use of this...
                //RenderTarget::from_framebuffer(context, width, height, framebuffer)

                // Get the screen render target to be able to render something on the screen
                //frame_input.screen()
                scc
                    // Clear the color and depth of the screen render target
                    .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                    .render(
                        &camera,
                        sphere
                            .into_iter()
                            .chain(&cylinder)
                            .chain(&cube)
                            .chain(&axes)
                            .chain(&bounding_box_sphere)
                            .chain(&bounding_box_cube)
                            .chain(&bounding_box_cylinder),
                        &[&light0, &light1],
                    );

                // Returns default frame output to end the frame
                //FrameOutput::default()


                window.gl_swap_window();

                
                if !running {
                    // unsafe {
                    //     context.delete_program(program);
                    //     context.delete_vertex_array(vertex_array);
                    // }
                    
                }
            }
        }



    }


    //high-level triangle with three-d window
    /* 
    {
        //normal HL trignalge
        // Create a window (a canvas on web)
        let window = Window::new(WindowSettings {
            title: "Triangle!".to_string(),
            max_size: Some((1280, 720)),
            ..Default::default()
        })
        .unwrap();

        // Get the graphics context from the window
        let context = window.gl();

        // Create a camera
        let mut camera = Camera::new_perspective(
            window.viewport(),
            vec3(0.0, 0.0, 2.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            10.0,
        );

        // Create a CPU-side mesh consisting of a single colored triangle
        let positions = vec![
            vec3(0.5, -0.5, 0.0),  // bottom right
            vec3(-0.5, -0.5, 0.0), // bottom left
            vec3(0.0, 0.5, 0.0),   // top
        ];
        let colors = vec![
            Srgba::RED,   // bottom right
            Srgba::GREEN, // bottom left
            Srgba::BLUE,  // top
        ];
        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            colors: Some(colors),
            ..Default::default()
        };

        // Construct a model, with a default color material, thereby transferring the mesh data to the GPU
        let mut model = Gm::new(Mesh::new(&context, &cpu_mesh), ColorMaterial::default());

        // Add an animation to the triangle.
        model.set_animation(|time| Mat4::from_angle_y(radians(time * 0.005)));

        // Start the main render loop
        window.render_loop(
            move |frame_input| // Begin a new frame with an updated frame input
        {
            // Ensure the viewport matches the current window viewport which changes if the window is resized
            camera.set_viewport(frame_input.viewport);

            // Update the animation of the triangle
            model.animate(frame_input.accumulated_time as f32);

            let scc = RenderTarget::screen(&context, frame_input.viewport.width, frame_input.viewport.height);

            // Get the screen render target to be able to render something on the screen
            //frame_input.screen()
            scc
                // Clear the color and depth of the screen render target
                .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                // Render the triangle with the color material which uses the per vertex colors defined at construction
                .render(
                    &camera, &model, &[]
                );

            // Returns default frame output to end the frame
            FrameOutput::default()
        },
        );
    }
    */

    //high-level triangle with sdl
    /*
    {

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

        let window_size = window.size();

        let vp = Viewport {
            x: 0,
            y: 0,
            width: window_size.0,
            height: window_size.1,
        };

        // Create a camera
        let mut camera = Camera::new_perspective(
            vp, //window.viewport(),
            vec3(0.0, 0.0, 2.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            10.0,
        );

        // Create a CPU-side mesh consisting of a single colored triangle
        let positions = vec![
            vec3(0.5, -0.5, 0.0),  // bottom right
            vec3(-0.5, -0.5, 0.0), // bottom left
            vec3(0.0, 0.5, 0.0),   // top
        ];
        let colors = vec![
            Srgba::RED,   // bottom right
            Srgba::GREEN, // bottom left
            Srgba::BLUE,  // top
        ];
        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            colors: Some(colors),
            ..Default::default()
        };

        // Construct a model, with a default color material, thereby transferring the mesh data to the GPU
        let mut model = Gm::new(Mesh::new(&context, &cpu_mesh), ColorMaterial::default());

        // Add an animation to the triangle.
        model.set_animation(|time| Mat4::from_angle_y(radians(time * 0.005)));

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


                unsafe {
                    context.clear_color(0.0, 0.0, 0.0, 1.0);
                    context.clear(context::COLOR_BUFFER_BIT | context::DEPTH_BUFFER_BIT);
                    //context.bind_buffer(target, buffer);
                    //context.set_blend(blend);
                    //context.bind_framebuffer(context::FRAMEBUFFER, Some(32));
                }
                
                // Ensure the viewport matches the current window viewport which changes if the window is resized
                camera.set_viewport(vp);//(frame_input.viewport);

                // Update the animation of the triangle
                time += 1.0;
                model.animate(time); //(frame_input.accumulated_time as f32);

                let scc = RenderTarget::screen(&context, vp.width, vp.height);

                // Get the screen render target to be able to render something on the screen
                //frame_input.screen()
                scc
                    // Clear the color and depth of the screen render target
                    .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                    // Render the triangle with the color material which uses the per vertex colors defined at construction
                    .render(
                        &camera, &model, &[]
                    );

                // Returns default frame output to end the frame
                //FrameOutput::default()


                window.gl_swap_window();

                
                if !running {
                    // unsafe {
                    //     context.delete_program(program);
                    //     context.delete_vertex_array(vertex_array);
                    // }
                    
                }
            }
        }



    }
    */


    //low-level triangle
    /* 
    return;
    {
        //original "window" object before we tried out SDL2
        // Create a window (a canvas on web)
        // let window2 = Window::new(WindowSettings {
        //     title: "Core Triangle!".to_string(),
        //     #[cfg(not(target_arch = "wasm32"))]
        //     max_size: Some((1280, 720)),
        //     ..Default::default()
        // })
        // .unwrap();
        // let ctt = window2.gl();
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

        let window_size = window.size();

        let vp = Viewport {
            x: 0,
            y: 0,
            width: window_size.0,
            height: window_size.1,
        };

        //hello triangle demo
        
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


                unsafe {
                    context.clear_color(0.0, 0.0, 0.0, 1.0);
                    context.clear(context::COLOR_BUFFER_BIT | context::DEPTH_BUFFER_BIT);
                    //context.bind_buffer(target, buffer);
                    //context.set_blend(blend);
                    //context.bind_framebuffer(context::FRAMEBUFFER, Some(32));
                }
                

                //hello triangle demo
                time += 1.0; //frame_input.accumulated_time as f32;
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

                

                if !running {
                    // unsafe {
                    //     context.delete_program(program);
                    //     context.delete_vertex_array(vertex_array);
                    // }
                    
                }
            }
        }


    }
    */





}
