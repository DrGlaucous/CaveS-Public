// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

use three_d::*;
use three_d_asset::io::*;

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "PBR!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(-3.0, 1.0, 2.5),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);
    let mut gui = three_d::GUI::new(&context);



    //DamagedHelmet_re.glb
    //untitled
    let mut loaded: RawAssets = three_d_asset::io::load(&[
        "C:/Users/EdwardStuckey/Documents/GitHub/CaveS-Public/Dim3/three-d-master/examples/assets/pano4s.png", // Source: https://polyhaven.com/
        "C:/Users/EdwardStuckey/Documents/GitHub/CaveS-Public/Dim3/three-d-master/examples/assets/gltf/untitled.glb", // Source: https://github.com/KhronosGroup/glTF-Sample-Models/tree/master/2.0
    ]).unwrap();

    //let environment_map: CpuTexture = loaded.deserialize("pano4s").unwrap();
    //let blank_map = CpuTexture::default();
    //blank_map.

    //let skybox = Skybox::new_from_equirectangular(&context, &environment_map);
    // let skybox = Skybox::new(&context,
    //     &blank_map, 
    //     &blank_map,
    //     &blank_map,
    //     &blank_map, 
    //     &blank_map, 
    //     &blank_map, 
    // );

    //note: with textures, this takes waaay longer.
    //potential solution: load the next map in the background while the current map is still running
    //you'd have to have an extra trigger and command to start the preload, then a copy command
    //and a wait-for-load-finish command that blocks until 


    let mut cpu_model: CpuModel = loaded.deserialize("untitled").unwrap();

    cpu_model
        .geometries
        .iter_mut()
        .for_each(|m| m.compute_normals());

    cpu_model
        .geometries
        .iter_mut()
        .for_each(|m| m.compute_tangents());

    let other_model = cpu_model.clone();


    let model = Model::<PhysicalMaterial>::new(&context, &cpu_model)
        .unwrap()
        ;//.remove(1);

    //let ambient = AmbientLight::new_with_environment(&context, 1.0, Srgba::WHITE, skybox.texture());
    let ambient = AmbientLight::new(&context, 1.0, Srgba::WHITE);

    let mut spot0 = PointLight::new(
        &context,
        0.0,
        Srgba::BLUE,
        &vec3(0.0, 0.0, 0.0),
        Attenuation { constant: 1.0, linear: 1.0, quadratic: 1.0 },
    );

    // main loop
    let mut normal_map_enabled = true;
    let mut occlusion_map_enabled = true;
    let mut metallic_roughness_enabled = true;
    let mut albedo_map_enabled = true;
    let mut emissive_map_enabled = true;
    window.render_loop(move |mut frame_input| {
        let mut panel_width = 0.0;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                use three_d::egui::*;
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Debug Panel");
                    ui.checkbox(&mut albedo_map_enabled, "Albedo map");
                    ui.checkbox(&mut metallic_roughness_enabled, "Metallic roughness map");
                    ui.checkbox(&mut normal_map_enabled, "Normal map");
                    ui.checkbox(&mut occlusion_map_enabled, "Occlusion map");
                    ui.checkbox(&mut emissive_map_enabled, "Emissive map");
                });
                panel_width = gui_context.used_rect().width();
            },
        );

        let viewport = Viewport {
            x: (panel_width * frame_input.device_pixel_ratio) as i32,
            y: 0,
            width: frame_input.viewport.width
                - (panel_width * frame_input.device_pixel_ratio) as u32,
            height: frame_input.viewport.height,
        };
        camera.set_viewport(viewport);
        control.handle_events(&mut camera, &mut frame_input.events);


        //let mut renderable_things = model.into_iter().chain(&skybox);

        let lights = [
            &ambient as &dyn Light,
            &spot0,
        ];

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.5, 0.5, 0.5, 1.0, 1.0))
            //.render(&camera, &skybox, &[])
            .render(&camera, &model, &lights)
            //.write(|| {
                /*
                let material = PhysicalMaterial {
                    name: model.material.name.clone(),
                    albedo: model.material.albedo,
                    albedo_texture: if albedo_map_enabled {
                        model.material.albedo_texture.clone()
                    } else {
                        None
                    },
                    metallic: model.material.metallic,
                    roughness: model.material.roughness,
                    metallic_roughness_texture: if metallic_roughness_enabled {
                        model.material.metallic_roughness_texture.clone()
                    } else {
                        None
                    },
                    normal_scale: model.material.normal_scale,
                    normal_texture: if normal_map_enabled {
                        model.material.normal_texture.clone()
                    } else {
                        None
                    },
                    occlusion_strength: model.material.occlusion_strength,
                    occlusion_texture: if occlusion_map_enabled {
                        model.material.occlusion_texture.clone()
                    } else {
                        None
                    },
                    emissive: if emissive_map_enabled {
                        model.material.emissive
                    } else {
                        Srgba::BLACK
                    },
                    emissive_texture: if emissive_map_enabled {
                        model.material.emissive_texture.clone()
                    } else {
                        None
                    },
                    render_states: model.material.render_states,
                    is_transparent: model.material.is_transparent,
                    lighting_model: LightingModel::Cook(
                        NormalDistributionFunction::TrowbridgeReitzGGX,
                        GeometryFunction::SmithSchlickGGX,
                    ),
                }; */

                //model.render_with_material(&material, &camera, &[&light]);
                //model.render_with_material(&material, &camera, &[&light]);
                //gui.render()
            //})
            ;//.unwrap();

        FrameOutput::default()
    });
}
