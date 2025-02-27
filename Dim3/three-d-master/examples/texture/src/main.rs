// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

use std::borrow::Borrow;

use three_d::*;

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Texture!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();


    //let aaaaaaa = Program::from_source(&context, )


    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(4.0, 1.5, 4.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let mut loaded = three_d_asset::io::load_async(&[
        "./../../examples/assets/skybox_evening/right.jpg",
        "./../../examples/assets/skybox_evening/left.jpg",
        "./../../examples/assets/skybox_evening/top.jpg",
        "./../../examples/assets/skybox_evening/front.jpg",
        "./../../examples/assets/skybox_evening/back.jpg",
        "./../../examples/assets/Skybox_example.png",
        "./../../examples/assets/PenguinBaseMesh.obj",
        "./../../examples/assets/marslike_big.jpg",
    ])
    .await
    .unwrap();

    // Skybox
    // let top_tex = loaded.deserialize("top").unwrap();
    // let skybox = Skybox::new(
    //     &context,
    //     &loaded.deserialize("right").unwrap(),
    //     &loaded.deserialize("left").unwrap(),
    //     &top_tex,
    //     &top_tex,
    //     &loaded.deserialize("front").unwrap(),
    //     &loaded.deserialize("back").unwrap(),
    // );
    let skybox = Skybox::new_from_equirectangular(
        &context,
        &loaded.deserialize("marslike_big").unwrap(),
    );

    // Box
    let mut cpu_texture: CpuTexture = loaded.deserialize("Skybox_example").unwrap();
    cpu_texture.data.to_linear_srgb();


    // let mut texx = Texture2D::new_empty::<[u8; 4]>(
    //     &context,
    //     200,
    //     200,
    //     Interpolation::Linear,
    //     Interpolation::Linear,
    //     None,
    //     Wrapping::ClampToEdge,
    //     Wrapping::ClampToEdge
    // );
    //texx.as_color_target(None).clear(ClearState::color(1.0, 0.8, 1.0, 1.0));

    //let mut apple = Texture2DRef::from_texture(texx);
    //apple.texture.borrow_mut();//as_color_target(None).clear(ClearState::color(1.0, 0.0, 1.0, 1.0));
    //apple.borrow.as_color_target(None).clear(ClearState::color(1.0, 0.0, 1.0, 1.0));




    let mut box_object = Gm::new(
        Mesh::new(&context, &CpuMesh::cube()),
        ColorMaterial {
            //texture: Some(Texture2DRef::from_texture(texx)),
            texture: Some(Texture2DRef::from_cpu_texture(&context, &cpu_texture)),
            ..Default::default()
        },
    );

    // if let Some(mut txx) = box_object.material.texture {
    //     txx.as_color_target(None).clear(ClearState::color(1.0, 0.0, 1.0, 1.0));
    // }



    box_object.material.render_states.cull = Cull::Back;

    // Penguin
    let model = loaded.deserialize("PenguinBaseMesh.obj").unwrap();
    let mut penguin = Model::<PhysicalMaterial>::new(&context, &model).unwrap();
    penguin.iter_mut().for_each(|m| {
        m.set_transformation(Mat4::from_translation(vec3(0.0, 1.0, 0.5)));
        m.material.render_states.cull = Cull::Back;
    });


    let mut ball_model = Gm::new(
        Mesh::new(&context, &CpuMesh::sphere(32)),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                roughness: 0.2,
                metallic: 0.8,
                ..Default::default()
            },
        ),
    );

    // Lights
    let ambient = AmbientLight::new_with_environment(&context, 1.0, Srgba::WHITE, skybox.texture());
    //let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let directional = DirectionalLight::new(&context, 2.0, Srgba::WHITE, &vec3(0.0, -1.0, -1.0));

    // main loop
    window.render_loop(move |mut frame_input| {
        let mut redraw = frame_input.first_frame;
        redraw |= camera.set_viewport(frame_input.viewport);
        redraw |= control.handle_events(&mut camera, &mut frame_input.events);

        // draw
        if redraw {
            frame_input.screen().clear(ClearState::default()).render(
                &camera,
                skybox.into_iter().chain(&ball_model), //penguin.into_iter().chain(&box_object).chain(&skybox),
                &[&ambient, &directional],
            );
        }

        FrameOutput {
            swap_buffers: redraw,
            ..Default::default()
        }
    });
}
