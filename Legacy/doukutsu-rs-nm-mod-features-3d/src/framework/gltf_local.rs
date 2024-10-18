use std::f32::consts::PI;
//use gltf::json::extensions::scene::khr_lights_punctual::Spot;
use gltf::scene::Transform;

use three_d::Attenuation;
//use three_d_asset::{animation::*, geometry::*, io::*, material::*, Error, Node, Result, Scene};
use three_d_asset::*;
use three_d_asset::io::*;
use three_d::renderer::light::{PointLight, SpotLight, DirectionalLight, Light as ThreeLight};
use three_d::core::Context;


use ::gltf::Gltf;
use gltf::khr_lights_punctual::{Light, Kind};
use std::collections::HashSet;
//use std::f32::consts::PI;
use std::path::{Path, PathBuf};

//noted GLTF import problems:
//lights are not supported (should be trivial to add, but they need to work with "scene" objects)
//exported models set their origin to the world center


pub fn dependencies(raw_assets: &RawAssets, path: &PathBuf) -> HashSet<PathBuf> {
    let mut dependencies = HashSet::new();
    if let Ok(Gltf { document, .. }) = Gltf::from_slice(raw_assets.get(path).unwrap()) {
        let base_path = path.parent().unwrap_or(Path::new(""));
        for buffer in document.buffers() {
            match buffer.source() {
                ::gltf::buffer::Source::Uri(uri) => {
                    if uri.starts_with("data:") {
                        dependencies.insert(PathBuf::from(uri));
                    } else {
                        dependencies.insert(base_path.join(uri));
                    }
                }
                _ => {}
            };
        }

        for texture in document.textures() {
            match texture.source().source() {
                ::gltf::image::Source::Uri { uri, .. } => {
                    if uri.starts_with("data:") {
                        use std::str::FromStr;
                        dependencies.insert(PathBuf::from_str(uri).unwrap());
                    } else {
                        dependencies.insert(base_path.join(uri));
                    }
                }
                _ => {}
            };
        }
    }
    dependencies
}

pub fn deserialize_gltf(context: &Context, raw_assets: &mut RawAssets, path: &PathBuf) -> Result<(Scene, Vec<Box<dyn ThreeLight>>)> {
    let Gltf { document, mut blob } = Gltf::from_slice(&raw_assets.remove(path)?)?;
    let base_path = path.parent().unwrap_or(Path::new(""));

    let mut buffers = Vec::new();
    for buffer in document.buffers() {
        let mut data = match buffer.source() {
            ::gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    raw_assets.remove(uri)?
                } else {
                    raw_assets.remove(base_path.join(uri))?
                }
            }
            ::gltf::buffer::Source::Bin => blob.take().ok_or(Error::GltfMissingData)?,
        };
        if data.len() < buffer.length() {
            Err(Error::GltfCorruptData)?;
        }
        while data.len() % 4 != 0 {
            data.push(0);
        }
        buffers.push(::gltf::buffer::Data(data));
    }

    let mut materials = Vec::new();
    for material in document.materials() {
        if let Some(_) = material.index() {
            materials.push(parse_material(
                raw_assets,
                &base_path,
                &mut buffers,
                &material,
            )?);
        }
    }


    //new: parse lights
    let mut light_vec: Vec<Box<dyn ThreeLight>> = Vec::new();
    // if let Some(light_iter) = document.lights() {
    //     for light in light_iter {
    //         light_vec.push(parse_lights(context, light));
    //     } 
    // }

    
    let mut nodes = Vec::new();
    for gltf_node in document.nodes() {
        let transformation = parse_transform(gltf_node.transform());
        // glTF say that if the scale is all zeroes, the node should be ignored.
        if let Some(light) = gltf_node.light() {
            //parse lights (we need to do it here so we can get the light transform)
            let light_transform = gltf_node.transform();
            light_vec.push(parse_lights(context, light, light_transform));

            nodes.push(None); //todo: is this the best thing to do?

        } else if transformation.determinant() != 0.0 {
            let name = gltf_node
                .name()
                .map(|s| s.to_string())
                .unwrap_or(format!("index {}", gltf_node.index()));
            let children = if let Some(mesh) = gltf_node.mesh() {
                parse_model(&mesh, &buffers)?
            } else {
                Vec::new()
            };

            
            nodes.push(Some(Node {
                name,
                transformation,
                children,
                ..Default::default()
            }));
        } else {
            nodes.push(None);
        }
    }

    for animation in document.animations() {
        let mut key_frames = Vec::new();
        let mut loop_time = 0.0f32;
        for channel in animation.channels() {
            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
            let interpolation = match channel.sampler().interpolation() {
                ::gltf::animation::Interpolation::Step => Interpolation::Nearest,
                ::gltf::animation::Interpolation::Linear => Interpolation::Linear,
                ::gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
            };
            let target_node = channel.target().node().index();
            let key = (
                target_node,
                channel.sampler().input().index(),
                interpolation,
            );
            let i = key_frames
                .iter_mut()
                .position(|(_, k, _)| k == &key)
                .unwrap_or_else(|| {
                    let times = reader.read_inputs().unwrap().collect::<Vec<_>>();
                    loop_time = loop_time.max(*times.last().unwrap_or(&0.0));
                    key_frames.push((
                        target_node,
                        key,
                        (
                            animation.name().map(|s| s.to_owned()),
                            KeyFrames {
                                times,
                                interpolation,
                                ..Default::default()
                            },
                        ),
                    ));
                    key_frames.len() - 1
                });
            let kf = &mut key_frames[i].2 .1;

            match reader.read_outputs().unwrap() {
                ::gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                    kf.rotations = Some(
                        rotations
                            .into_f32()
                            .into_iter()
                            .map(|r| Quat::from_sv(r[3], vec3(r[0], r[1], r[2])))
                            .collect(),
                    );
                }
                ::gltf::animation::util::ReadOutputs::Translations(translations) => {
                    kf.translations = Some(
                        translations
                            .into_iter()
                            .map(|r| vec3(r[0], r[1], r[2]))
                            .collect(),
                    );
                }
                ::gltf::animation::util::ReadOutputs::Scales(scales) => {
                    kf.scales = Some(scales.into_iter().map(|r| vec3(r[0], r[1], r[2])).collect());
                }
                ::gltf::animation::util::ReadOutputs::MorphTargetWeights(weights) => {
                    let weights = weights.into_f32().collect::<Vec<_>>();
                    let count = weights.len() / kf.times.len();
                    kf.weights = Some(
                        weights
                            .chunks(count)
                            .map(|c| c.into_iter().map(|v| *v).collect::<Vec<_>>())
                            .collect(),
                    );
                }
            }
        }
        for (target_node, _, mut kf) in key_frames {            
            nodes[target_node].as_mut().map(|n| {
                kf.1.loop_time = Some(loop_time);
                n.animations.push(kf);
                //new: reset position since animations are relative to origin
                n.transformation = Mat4::identity();
            });
        }
    }

    let gltf_scene = document.scenes().nth(0).unwrap();
    let mut scene = Scene {
        name: gltf_scene
            .name()
            .unwrap_or(&format!("Scene {}", gltf_scene.index()))
            .to_owned(),
        materials,
        children: Vec::new(),
    };
    for c in gltf_scene.nodes() {
        if let Some(mut node) = nodes[c.index()].take() {
            visit(c, &mut nodes, &mut node.children);
            scene.children.push(node);
        }
    }



    Ok((scene, light_vec))
}

fn visit(gltf_node: ::gltf::Node, nodes: &mut Vec<Option<Node>>, children: &mut Vec<Node>) {
    for c in gltf_node.children() {
        if let Some(mut node) = nodes[c.index()].take() {
            visit(c, nodes, &mut node.children);
            children.push(node);
        }
    }
}

fn parse_model(mesh: &::gltf::mesh::Mesh, buffers: &[::gltf::buffer::Data]) -> Result<Vec<Node>> {
    let mut children = Vec::new();
    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        if let Some(read_positions) = reader.read_positions() {
            let positions: Vec<_> = read_positions.map(|p| p.into()).collect();

            let normals = reader
                .read_normals()
                .map(|values| values.map(|n| n.into()).collect());

            let tangents = reader
                .read_tangents()
                .map(|values| values.map(|t| t.into()).collect());

            let indices = reader
                .read_indices()
                .map(|values| match values {
                    ::gltf::mesh::util::ReadIndices::U8(iter) => Indices::U8(iter.collect()),
                    ::gltf::mesh::util::ReadIndices::U16(iter) => Indices::U16(iter.collect()),
                    ::gltf::mesh::util::ReadIndices::U32(iter) => Indices::U32(iter.collect()),
                })
                .unwrap_or(Indices::None);

            let colors = reader.read_colors(0).map(|values| {
                values
                    .into_rgba_u8()
                    .map(|c| Srgba::new(c[0], c[1], c[2], c[3]))
                    .collect()
            });

            let uvs = reader
                .read_tex_coords(0)
                .map(|values| values.into_f32().map(|uv| uv.into()).collect());

            children.push(Node {
                geometry: Some(Geometry::Triangles(TriMesh {
                    positions: Positions::F32(positions),
                    normals,
                    tangents,
                    indices,
                    colors,
                    uvs,
                })),
                material_index: primitive.material().index(),
                ..Default::default()
            });
        }
    }
    Ok(children)
}

fn material_name(material: &::gltf::material::Material) -> String {
    material.name().map(|s| s.to_string()).unwrap_or(
        material
            .index()
            .map(|i| format!("index {}", i))
            .unwrap_or("default".to_string()),
    )
}

fn parse_material(
    raw_assets: &mut RawAssets,
    path: &Path,
    buffers: &[::gltf::buffer::Data],
    material: &::gltf::material::Material,
) -> Result<PbrMaterial> {
    let pbr = material.pbr_metallic_roughness();
    let color = pbr.base_color_factor();
    let albedo_texture = if let Some(info) = pbr.base_color_texture() {
        Some(parse_texture(raw_assets, path, buffers, info.texture())?)
    } else {
        None
    };
    let metallic_roughness_texture = if let Some(info) = pbr.metallic_roughness_texture() {
        Some(parse_texture(raw_assets, path, buffers, info.texture())?)
    } else {
        None
    };
    let (normal_texture, normal_scale) = if let Some(normal) = material.normal_texture() {
        (
            Some(parse_texture(raw_assets, path, buffers, normal.texture())?),
            normal.scale(),
        )
    } else {
        (None, 1.0)
    };
    let (occlusion_texture, occlusion_strength) =
        if let Some(occlusion) = material.occlusion_texture() {
            (
                Some(parse_texture(
                    raw_assets,
                    path,
                    buffers,
                    occlusion.texture(),
                )?),
                occlusion.strength(),
            )
        } else {
            (None, 1.0)
        };
    let emissive_texture = if let Some(info) = material.emissive_texture() {
        Some(parse_texture(raw_assets, path, buffers, info.texture())?)
    } else {
        None
    };
    let transmission_texture =
        if let Some(Some(info)) = material.transmission().map(|t| t.transmission_texture()) {
            Some(parse_texture(raw_assets, path, buffers, info.texture())?)
        } else {
            None
        };
    Ok(PbrMaterial {
        name: material_name(material),
        albedo: color.into(),
        albedo_texture,
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        metallic_roughness_texture,
        normal_texture,
        normal_scale,
        occlusion_texture,
        occlusion_strength,
        occlusion_metallic_roughness_texture: None,
        emissive: material.emissive_factor().into(),
        emissive_texture,
        transmission: material
            .transmission()
            .map(|t| t.transmission_factor())
            .unwrap_or(0.0),
        transmission_texture,
        index_of_refraction: material.ior().unwrap_or(1.5),
        alpha_cutout: material.alpha_cutoff(),
        lighting_model: LightingModel::Cook(
            NormalDistributionFunction::TrowbridgeReitzGGX,
            GeometryFunction::SmithSchlickGGX,
        ),
    })
}

// impl Into<Wrapping> for gltf::texture::WrappingMode {
//     fn into(self) -> Wrapping {
//         match self {
//             ::gltf::texture::WrappingMode::ClampToEdge => Wrapping::ClampToEdge,
//             ::gltf::texture::WrappingMode::MirroredRepeat => Wrapping::MirroredRepeat,
//             ::gltf::texture::WrappingMode::Repeat => Wrapping::Repeat,
//         }
//     }
// }

fn parse_texture<'a>(
    raw_assets: &mut RawAssets,
    path: &Path,
    buffers: &[::gltf::buffer::Data],
    gltf_texture: ::gltf::texture::Texture,
) -> Result<Texture2D> {
    let gltf_image = gltf_texture.source();
    let gltf_source = gltf_image.source();
    let mut tex: Texture2D = match gltf_source {
        ::gltf::image::Source::Uri { uri, .. } => {
            if uri.starts_with("data:") {
                raw_assets.deserialize(uri)?
            } else {
                raw_assets.deserialize(path.join(uri))?
            }
        }
        ::gltf::image::Source::View { view, .. } => {
            if view.stride() != None {
                unimplemented!();
            }
            #[allow(unused_variables)]
            let buffer = &buffers[view.buffer().index()];
            #[cfg(not(feature = "image"))]
            return Err(Error::FeatureMissing("image".to_string()));
            #[cfg(feature = "image")]
            super::img::deserialize_img("", &buffer[view.offset()..view.offset() + view.length()])?
        }
    };

    let sampler = gltf_texture.sampler();
    tex.mag_filter = match sampler.mag_filter() {
        Some(::gltf::texture::MagFilter::Nearest) => Interpolation::Nearest,
        Some(::gltf::texture::MagFilter::Linear) => Interpolation::Linear,
        None => tex.mag_filter,
    };
    (tex.min_filter, tex.mip_map_filter) = match sampler.min_filter() {
        Some(::gltf::texture::MinFilter::Nearest) => (Interpolation::Nearest, None),
        Some(::gltf::texture::MinFilter::Linear) => (Interpolation::Linear, None),
        Some(::gltf::texture::MinFilter::NearestMipmapNearest) => {
            (Interpolation::Nearest, Some(Interpolation::Nearest))
        }
        Some(::gltf::texture::MinFilter::LinearMipmapNearest) => {
            (Interpolation::Linear, Some(Interpolation::Nearest))
        }
        Some(::gltf::texture::MinFilter::NearestMipmapLinear) => {
            (Interpolation::Nearest, Some(Interpolation::Linear))
        }
        Some(::gltf::texture::MinFilter::LinearMipmapLinear) => {
            (Interpolation::Linear, Some(Interpolation::Linear))
        }
        None => (tex.min_filter, tex.mip_map_filter),
    };
    tex.wrap_s = sampler.wrap_s().into();
    tex.wrap_t = sampler.wrap_t().into();

    Ok(tex)
}

fn parse_transform(transform: ::gltf::scene::Transform) -> Mat4 {
    let [c0, c1, c2, c3] = transform.matrix();
    Mat4::from_cols(c0.into(), c1.into(), c2.into(), c3.into())
}


//todo: keyframe support for lights (we can do this later...)
fn parse_lights(context: &Context, doc_light: Light, light_transform: Transform) -> Box<dyn ThreeLight> {

    let (translation, rotation, scale) = light_transform.decomposed();

    let color = doc_light.color();
    let color = Srgba::from(color);

    let intensity = doc_light.intensity();

    //intensity is unitless with three-d, so we'll try to eyeball it compared to blender's intensity so we get close to the same results between softwares
    let point_multiplier = 1.0 / 30000.0;
    let sun_multiplier = 1.0 / 100.0;
    let spot_multiplier = 1.0 / 1000.0;

    //let position = doc_light.
    let position = Vector3 { x:translation[0], y: translation[1], z: translation[2] };
    let direction = quaternion_to_euler(Vector4::from(rotation));

    //print!("Light X:{} Y:{} Z:{}\n", position.x, position.y, position.z);
    //print!("ROTATION X:{} Y:{} Z:{}\n", direction.x, direction.y, direction.z);


    //todo: non-point-light direction is very broken
    //x axis rotates around blender "Y", 180 shines toward x+
    //
    //let direction = Vector3::new(90.0,0.0,0.0);

    let light: Box<dyn ThreeLight> = match doc_light.kind() {
        Kind::Point => {
            Box::new(PointLight::new(context, intensity * point_multiplier, color, &position, Attenuation::default()))
        }
        Kind::Directional => {
            Box::new(DirectionalLight::new(context, intensity * sun_multiplier, color, &direction))
        }
        Kind::Spot { inner_cone_angle, outer_cone_angle: _ } => {
            let cutoff = Rad(inner_cone_angle);
            //todo: use outer_cone_angle to find attenuation            
            Box::new(SpotLight::new(context, intensity * spot_multiplier, color, &position, &direction, cutoff, Attenuation::default()))
        }
    };

    light


}


fn quaternion_to_euler(input: Vector4<f32>) -> Vector3<f32> {

    let x = input.w; //x
    let y = input.x; //y
    let z = input.y; //z
    let w = input.z; //w

    // Roll (X-axis)
    let sinr_cosp = 2.0 * (w * x + y * z);
    let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
    let roll = sinr_cosp.atan2(cosr_cosp);

    // Pitch (Y-axis)
    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        // Handle gimbal lock
        sinp.signum() * PI / 2.0
    } else {
        sinp.asin()
    };

    // Yaw (Z-axis)
    let siny_cosp = 2.0 * (w * z + x * y);
    let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
    let yaw = siny_cosp.atan2(cosy_cosp);

    Vector3::from((roll, pitch, yaw))
    
}
