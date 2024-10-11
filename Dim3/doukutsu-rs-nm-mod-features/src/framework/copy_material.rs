use std::num::NonZeroU32;
use three_d::context::NativeTexture;
use three_d::core::*;
use three_d::renderer::*;

use super::gl::TEXTURE_2D;

///
/// A material that renders a [Geometry] in a color defined by multiplying a color with an optional texture and optional per vertex colors.
/// This material is not affected by lights.
///
#[derive(Clone)]
pub struct CopyMaterial {
    /// Base surface color.
    //pub color: Srgba,
    /// An optional texture which is samples using uv coordinates (requires that the [Geometry] supports uv coordinates).
    /// The colors are assumed to be in linear sRGB (`RgbU8`), linear sRGB with an alpha channel (`RgbaU8`) or HDR color space.
    //pub texture: Option<Texture2DRef>,
    
    /// Render states.
    pub render_states: RenderStates,
    
    /// Whether this material should be treated as a transparent material (An object needs to be rendered differently depending on whether it is transparent or opaque).
    pub is_transparent: bool,

    pub tex_id: NonZeroU32,
}

impl CopyMaterial {
    ///
    /// Constructs a new buffer material, which targets a pre-existing raw lower-level buffer directly inside openGL
    ///
    pub fn new(is_transparent: bool, tex_id: u32) -> Self {
        Self {
            render_states: RenderStates::default(),
            is_transparent,
            tex_id: NonZeroU32::new(tex_id).unwrap(),
        }
    }
}

impl Default for CopyMaterial {
    fn default() -> Self {
        Self {
            render_states: RenderStates::default(),
            is_transparent: bool::default(),
            tex_id: NonZeroU32::new(1).unwrap(),
        }
    }
}

impl Material for CopyMaterial {

    //EffectMaterialId is from a newer version: this uses u16
    fn id(&self) -> u16 {
        2
    }

    fn fragment_shader_source(&self, _lights: &[&dyn Light]) -> String {
        let mut shader = String::new();

        //default colored shader (it seems permanently washed out for some reason...)
        // shader.push_str("#define USE_TEXTURE\nin vec2 uvs;\n");
        // shader.push_str(include_str!("shaders/threed/shared.frag"));
        // shader.push_str(ColorMapping::fragment_shader_source());
        // shader.push_str(include_str!("shaders/threed/color_material.frag"));
        
        //trimmed out all the fat with this one
        shader.push_str(include_str!("shaders/threed/simple_c_shader.frag"));
        
        shader
    }

    fn fragment_attributes(&self) -> FragmentAttributes {

        //from the vertex shader, pass these two attributes into the fragment shader
        FragmentAttributes {
            color: true,
            uv: true,
            ..FragmentAttributes::NONE
        }
    }

    fn use_uniforms(&self, program: &Program, camera: &Camera, _lights: &[&dyn Light]) {
        
        //default stuff with original color shader
        //camera.color_mapping.use_uniforms(program);
        //program.use_uniform("surfaceColor", self.color.to_linear_srgb());


        let transform = Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
        program.use_uniform("textureTransformation", transform);

        //note: for stock d-rs, the "good stuff" seems to be on screen buffer 2
        let txx = NativeTexture(self.tex_id);
        program.use_raw_texture("tex", TEXTURE_2D, txx);

    }
    
    fn render_states(&self) -> RenderStates {
        self.render_states
    }
    
    fn material_type(&self) -> MaterialType {
        if self.is_transparent {
            MaterialType::Transparent
        } else {
            MaterialType::Opaque
        }
    }
}
