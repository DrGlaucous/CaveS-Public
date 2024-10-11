// Entry point for non-wasm
// #[cfg(not(target_arch = "wasm32"))]
// #[tokio::main]
fn main() {
    run();
}

use three_d::*;

pub fn run() {
    let window = Window::new(WindowSettings {
        title: "Forest!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(100.0, 250.0, 100.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(60.0),
        0.1,
        10000.0,
    );
    let mut control = FlyControl::new(0.1);

    let mut loaded = three_d_asset::io::load(&["C:/Users/EdwardStuckey/Documents/GitHub/CaveS-Public/Dim3/meshes/cubeobj/cube.obj"]).unwrap();
    // let mut loaded = if let Ok(loaded) =
    //     three_d_asset::io::load_async(&["C:/Users/EdwardStuckey/Documents/GitHub/CaveS-Public/Dim3/meshes/Gledista_Triacanthos.obj"]).await
    // {
    //     loaded
    // } else {
    //     three_d_asset::io::load_async(&[
    //         "https://asny.github.io/three-d/assets/Gledista_Triacanthos.obj",
    //     ])
    //     .await
    //     .expect("failed to download the necessary assets, to enable running this example offline, place the relevant assets in a folder called 'assets' next to the three-d source")
    // };


    // Tree
    let mut cpu_model: CpuModel = loaded.deserialize(".obj").unwrap();
    cpu_model
        .geometries
        .iter_mut()
        .for_each(|g| g.compute_normals());
    let mut model = Model::<PhysicalMaterial>::new(&context, &cpu_model).unwrap();
    // model
    //     .iter_mut()
    //     .for_each(|m| m.material.render_states.cull = Cull::Back);

    // Lights
    let ambient = AmbientLight::new(&context, 0.3, Srgba::WHITE);
    let directional = DirectionalLight::new(&context, 4.0, Srgba::WHITE, &vec3(-1.0, -1.0, -1.0));

    // Imposters

    //make a bounding box that fits the size of the model
    let mut aabb = AxisAlignedBoundingBox::EMPTY;
    model.iter().for_each(|m| {
        aabb.expand_with_aabb(&m.aabb());
    });
    let size = aabb.size();
    //determine the positions of each 2d "imposter"
    let t = 100;
    let mut positions = Vec::new();
    for x in -t..t + 1 {
        for y in -t..t + 1 {
            if x != 0 || y != 0 {
                positions.push(vec3(size.x * x as f32, 0.0, size.y * y as f32));
            }
        }
    }

    let imposters = Imposters::new(&context, &positions, &model, &[&ambient, &directional], 256);

    // Plane
    let mut plane = Gm::new(
        Mesh::new(
            &context,
            &CpuMesh {
                positions: Positions::F32(vec![
                    vec3(-10000.0, 0.0, 10000.0),
                    vec3(10000.0, 0.0, 10000.0),
                    vec3(0.0, 0.0, -10000.0),
                ]),
                normals: Some(vec![
                    vec3(0.0, 1.0, 0.0),
                    vec3(0.0, 1.0, 0.0),
                    vec3(0.0, 1.0, 0.0),
                ]),
                ..Default::default()
            },
        ),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba::new_opaque(128, 200, 70),
                metallic: 0.0,
                roughness: 1.0,
                ..Default::default()
            },
        ),
    );
    //plane.material.render_states.cull = Cull::Back;

    // main loop
    window.render_loop(move |mut frame_input| {
        let mut redraw = frame_input.first_frame;
        redraw |= camera.set_viewport(frame_input.viewport);

        redraw |= control.handle_events(&mut camera, &mut frame_input.events);

        if redraw {
            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                .render(
                    &camera,
                    model.into_iter().chain(&imposters).chain(&plane),
                    &[&ambient, &directional],
                );
        }

        FrameOutput {
            swap_buffers: redraw,
            ..Default::default()
        }
    });
}
