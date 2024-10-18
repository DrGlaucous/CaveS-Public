use crate::common::{Color, Rect};
use crate::framework::backend::{BackendShader, BackendTexture, VertexData};
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::game::Game;
use crate::graphics;

use super::render_opengl::{OpenGLRenderer, ThreeDModelSetup};

pub enum FilterMode {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    None,
    /// When combining two fragments, add their values together, saturating
    /// at 1.0
    Add,
    /// When combining two fragments, add the value of the source times its
    /// alpha channel with the value of the destination multiplied by the inverse
    /// of the source alpha channel. Has the usual transparency effect: mixes the
    /// two colors using a fraction of each one specified by the alpha of the source.
    Alpha,
    /// When combining two fragments, multiply their values together.
    Multiply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VSyncMode {
    /// No V-Sync - uncapped frame rate
    Uncapped,
    /// Synchronized to V-Sync
    VSync,
    /// Variable Refresh Rate - Synchronized to game tick interval
    VRRTickSync1x,
    /// Variable Refresh Rate - Synchronized to 2 * game tick interval
    VRRTickSync2x,
    /// Variable Refresh Rate - Synchronized to 3 * game tick interval
    VRRTickSync3x,
}

pub fn clear(ctx: &mut Context, color: Color) {
    if let Some(renderer) = &mut ctx.renderer {
        renderer.clear(color)
    }
}

pub fn present(ctx: &mut Context) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        renderer.present()?;
    }

    Ok(())
}

pub fn set_vsync_mode(ctx: &mut Context, mode: VSyncMode) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        ctx.vsync_mode = mode;
        renderer.set_vsync_mode(mode);
    }

    Ok(())
}

#[allow(unused)]
pub fn renderer_initialized(ctx: &mut Context) -> bool {
    ctx.renderer.is_some()
}

pub fn create_texture_mutable(ctx: &mut Context, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.create_texture_mutable(width, height);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn create_texture(ctx: &mut Context, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.create_texture(width, height, data);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn screen_size(ctx: &mut Context) -> (f32, f32) {
    ctx.screen_size
}

#[allow(unused)]
pub fn screen_insets(ctx: &mut Context) -> (f32, f32, f32, f32) {
    ctx.screen_insets
}

pub fn screen_insets_scaled(ctx: &mut Context, scale: f32) -> (f32, f32, f32, f32) {
    (ctx.screen_insets.0 / scale, ctx.screen_insets.1 / scale, ctx.screen_insets.2 / scale, ctx.screen_insets.3 / scale)
}

pub fn set_render_target(ctx: &mut Context, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.set_render_target(texture);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn set_blend_mode(ctx: &mut Context, blend: BlendMode) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.set_blend_mode(blend);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn draw_rect(ctx: &mut Context, rect: Rect, color: Color) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.draw_rect(rect, color);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

#[allow(unused)]
pub fn draw_outline_rect(ctx: &mut Context, rect: Rect, line_width: usize, color: Color) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.draw_outline_rect(rect, line_width, color);
    }

    Ok(())
}

pub fn set_clip_rect(ctx: &mut Context, rect: Option<Rect>) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.set_clip_rect(rect);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn imgui_context(ctx: &Context) -> GameResult<&mut imgui::Context> {
    if let Some(renderer) = ctx.renderer.as_ref() {
        return renderer.imgui();
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn imgui_texture_id(ctx: &Context, texture: &Box<dyn BackendTexture>) -> GameResult<imgui::TextureId> {
    if let Some(renderer) = ctx.renderer.as_ref() {
        return renderer.imgui_texture_id(texture);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn prepare_imgui(ctx: &mut Context, ui: &imgui::Ui) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.prepare_imgui(ui);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn render_imgui(ctx: &mut Context, draw_data: &imgui::DrawData) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.render_imgui(draw_data);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn prepare_draw(ctx: &mut Context) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.prepare_draw(ctx.screen_size.0, ctx.screen_size.1);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn supports_vertex_draw(ctx: &Context) -> GameResult<bool> {
    if let Some(renderer) = ctx.renderer.as_ref() {
        return Ok(renderer.supports_vertex_draw());
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn draw_triangle_list(
    ctx: &mut Context,
    vertices: &[VertexData],
    texture: Option<&Box<dyn BackendTexture>>,
    shader: BackendShader,
) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.draw_triangle_list(vertices, texture, shader);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

/////new 3d methods:

fn check_for_renderer(ctx: &mut Context) -> GameResult<&mut ThreeDModelSetup>{
    if let Some(renderer) = &mut ctx.renderer {

        let gl_renderer = renderer
            .as_any_mut()
            .downcast_mut::<OpenGLRenderer>();
        if let Some(renderer) = gl_renderer {
            if let Some(model) = &mut renderer.model {

                return Ok(model);
            }

            return Err(GameError::RenderError(format!("Three-d not initialized!")));
        }

        return Err(GameError::RenderError(format!("Renderer is not OpenGL!")));
    }

    return Err(GameError::RenderError(format!("Renderer is not initialized!")));
}

/// updates the screen size in the three-d context (only works with an openGL renderer)
pub fn set_3d_viewport(ctx: &mut Context, width: u32, height: u32, scale: f32) -> GameResult {


    let model = check_for_renderer(ctx).unwrap();
    model.set_viewport_size(width, height, scale);
    Ok(())

    // if let Some(renderer) = &mut ctx.renderer {
    //     let gl_renderer = renderer
    //         .as_any_mut()
    //         .downcast_mut::<OpenGLRenderer>();
    //     if let Some(renderer) = gl_renderer {
    //         if let Some(model) = &mut renderer.model {
    //             model.set_viewport_size(width, height, scale);
    //             return Ok(());
    //         }
    //         return Err(GameError::RenderError(format!("Three-d not initialized!")));
    //     }
    //     return Err(GameError::RenderError(format!("Renderer is not OpenGL!")));
    // }
    // return Err(GameError::RenderError(format!("Renderer is not initialized!")));

}

/// set the texture that will be used to draw on the 3d scene's char plane
pub fn set_3d_char_plane(ctx: &mut Context, source_texture: &Box<dyn BackendTexture>) -> GameResult {

    let model = check_for_renderer(ctx).unwrap();
    model.set_char_plane_target_surf(source_texture)?;
    Ok(())

}

/// draw the 3D scene to this texture
pub fn draw_3d(ctx: &mut Context, dest_texture: Option<&Box<dyn BackendTexture>>) -> GameResult {


    let model = check_for_renderer(ctx).unwrap();
    model.draw(dest_texture)?;

    //reset texture back to whatever we've bound it to, since three-d resets it to buffer 0 when done
    set_render_target(ctx, dest_texture)?;

    Ok(())
}

/// move the 3d scene to the location of the frame (units: meters)
pub fn update_frame_location(ctx: &mut Context, x: f32, y: f32) -> GameResult {

    let model = check_for_renderer(ctx).unwrap();
    model.set_location(x, y);

    Ok(())

}

/// attempt to load a gltf into the 3d scene
pub fn load_gltf(ctx: &mut Context, data: &[u8], key: i32, update_lights: bool) -> GameResult<bool> {
    let model = check_for_renderer(ctx).unwrap();
    model.load_gltf(data, key, update_lights)
}

/// delete the gltf loaded into this slot
pub fn unload_gltf(ctx: &mut Context, key: i32) -> GameResult<bool> {
    let model = check_for_renderer(ctx).unwrap();
    model.unload_gltf(key)
}

/// clear all currently loaded gltfs
pub fn clear_gltf(ctx: &mut Context) -> GameResult {
    let model = check_for_renderer(ctx).unwrap();
    model.clear_gltf();
    Ok(())
}

/// load a skybox into the scene, optionally setting the ambient light texture to this
pub fn load_skybox(ctx: &mut Context, data: &[u8], have_ambient: bool) -> GameResult {

    let model = check_for_renderer(ctx).unwrap();
    model.load_skybox(data, have_ambient)?;

    Ok(())
}

/// unload the skybox from the scene
pub fn unload_skybox(ctx: &mut Context) -> GameResult {

    let model = check_for_renderer(ctx).unwrap();
    model.unload_skybox();

    Ok(())
}

/// set optional light texture, color, or intensity of the ambient light
pub fn set_ambient_attributes(ctx: &mut Context, data: Option<&[u8]>, color: Option<Color>, intensity: Option<f32>) -> GameResult {
    let model = check_for_renderer(ctx).unwrap();
    model.set_ambient_attributes(data, color, intensity)?;

    Ok(())
}

/// unload the ambient light's texture
pub fn unload_ambient_image(ctx: &mut Context) -> GameResult {
    let model = check_for_renderer(ctx).unwrap();
    model.unload_ambient_image();

    Ok(())
}

/// increment backend animation time (in seconds, where 1 game tick is 1/50 seconds)
pub fn increment_animation_time(ctx: &mut Context, delta_time: f32, offset_time: f32) -> GameResult {
    let model = check_for_renderer(ctx).unwrap();
    model.increment_animation_time(delta_time, offset_time);

    Ok(())
}

/// increment backend animation time (in seconds, where 1 game tick is 1/50 seconds)
pub fn set_model_animation_attributes(ctx: &mut Context, key: i32, anim_name: Option<&str>, time: Option<f32>, play: Option<bool>, stop_time: Option<f32>) -> GameResult {
    let model = check_for_renderer(ctx).unwrap();

    if let Some(anim_name) = anim_name {
        model.set_model_animation(key, anim_name);
    }

    if let Some(time) = time {
        model.set_model_anim_time(key, time);

    }

    if let Some(play) = play {
        model.set_model_anim_state(key, play);
    }

    if let Some(stop_time) = stop_time {
        model.set_model_anim_stop_time(key, stop_time);
    }


    Ok(())
}
