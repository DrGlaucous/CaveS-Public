use std::collections::HashMap;
use std::num::NonZeroU32;
use std::any::Any;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::{c_void, CStr};
use std::hint::unreachable_unchecked;
use std::mem;
use std::mem::MaybeUninit;
use std::mem::ManuallyDrop;
use std::ptr::null;
use std::sync::Arc;
use std::path;

use context::NativeFramebuffer;
use three_d::*;
use three_d::core::Context as ThreeDContext;
use three_d::context::Context as GlowContext;
use three_d::context::NativeTexture;
use three_d::geometry::CpuMesh;


use imgui::{DrawCmd, DrawCmdParams, DrawData, DrawIdx, DrawVert, TextureId, Ui};
use three_d_asset::io::RawAssets;
use three_d_asset::ProjectionType;

use crate::common::{Color, Rect};
use crate::framework::backend::{BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand, VertexData};
use crate::framework::context::Context;
use crate::framework::error::GameError;
use crate::framework::error::GameError::RenderError;
use crate::framework::error::GameResult;
use crate::framework::gl;
use crate::framework::gl::types::*;
use crate::framework::graphics::{BlendMode, VSyncMode};
use crate::framework::util::{field_offset, return_param};
use crate::game::{Game, GAME_SUSPENDED};

use crate::framework::buffer_material::BufferMaterial;
use crate::framework::gltf_local::deserialize_gltf;


// fn chain_all<I>(items: &Vec<I>) -> impl Iterator<Item = I::Item>
// where
//     I: IntoIterator,
// {
//     items.into_iter()
//         .map(|item| item.into_iter())
//         .fold(std::iter::empty(), |acc, iter| acc.chain(iter))
// }


pub fn handle_err(gl: &Gl, extra_info: u32) {
    
    //extra_info = 0: nothing
    //1: pulled from load_gl (655)



    unsafe{
        let err = gl.gl.GetError();
        //gl::INVALID_ENUM
        if err != 0 && extra_info != 1 {
        //if err != 0 {
            log::error!("OpenGL error: {}", err);
        }
    }

}


pub struct GLContext {
    pub gles2_mode: bool,
    pub is_sdl: bool,
    pub get_proc_address: unsafe fn(user_data: &mut *mut c_void, name: &str) -> *const c_void,
    pub swap_buffers: unsafe fn(user_data: &mut *mut c_void),
    pub user_data: *mut c_void,
    pub ctx: *mut Context,
}

pub struct OpenGLTexture {
    width: u16,
    height: u16,
    texture_id: u32,
    framebuffer_id: u32,
    shader: RenderShader,
    vbo: GLuint,
    vertices: Vec<VertexData>,
    context_active: Arc<RefCell<bool>>,
}

impl BackendTexture for OpenGLTexture {
    fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn add(&mut self, command: SpriteBatchCommand) {
        let (tex_scale_x, tex_scale_y) = (1.0 / self.width as f32, 1.0 / self.height as f32);

        match command {
            SpriteBatchCommand::DrawRect(src, dest) => {
                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectFlip(mut src, dest, flip_x, flip_y) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }

                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }

                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectTinted(src, dest, color) => {
                let color = color.to_rgba();
                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectFlipTinted(mut src, dest, flip_x, flip_y, color) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }

                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }

                let color = color.to_rgba();

                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
        }
    }

    fn clear(&mut self) {
        self.vertices.clear();
    }

    fn draw(&mut self) -> GameResult {
        unsafe {
            if let Some(gl) = &GL_PROC {

                handle_err(gl, 0);

                if self.texture_id == 0 {
                    return Ok(());
                }

                if gl.gl.BindSampler.is_loaded() {
                    gl.gl.BindSampler(0, 0);
                }

                gl.gl.Enable(gl::TEXTURE_2D);
                gl.gl.Enable(gl::BLEND);
                gl.gl.Disable(gl::DEPTH_TEST);

                self.shader.bind_attrib_pointer(gl, self.vbo);

                gl.gl.BindTexture(gl::TEXTURE_2D, self.texture_id);
                gl.gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.len() * mem::size_of::<VertexData>()) as _,
                    self.vertices.as_ptr() as _,
                    gl::STREAM_DRAW,
                );

                gl.gl.DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as _);

                gl.gl.BindTexture(gl::TEXTURE_2D, 0);
                gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);

                handle_err(gl, 0);

                Ok(())
            } else {
                Err(RenderError("No OpenGL context available!".to_string()))
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for OpenGLTexture {
    fn drop(&mut self) {
        if *self.context_active.as_ref().borrow() {
            unsafe {
                if let Some(gl) = &GL_PROC {
                    if self.texture_id != 0 {
                        let texture_id = &self.texture_id;
                        gl.gl.DeleteTextures(1, texture_id as *const _);
                    }

                    if self.framebuffer_id != 0 {}

                    handle_err(gl, 0);
                }
            }
        }
    }
}

fn check_shader_compile_status(shader: u32, gl: &Gl) -> GameResult {
    unsafe {
        let mut status: GLint = 0;
        gl.gl.GetShaderiv(shader, gl::COMPILE_STATUS, (&mut status) as *mut _);

        if status == (gl::FALSE as GLint) {
            let mut max_length: GLint = 0;
            let mut msg_length: GLsizei = 0;
            gl.gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, (&mut max_length) as *mut _);

            let mut data: Vec<u8> = vec![0; max_length as usize];
            gl.gl.GetShaderInfoLog(
                shader,
                max_length as GLsizei,
                (&mut msg_length) as *mut _,
                data.as_mut_ptr() as *mut _,
            );

            let data = String::from_utf8_lossy(&data);
            return Err(GameError::RenderError(format!("Failed to compile shader {}: {}", shader, data)));
        }
    }

    Ok(())
}

const VERTEX_SHADER_BASIC: &str = include_str!("shaders/opengl/vertex_basic_110.glsl");
const FRAGMENT_SHADER_TEXTURED: &str = include_str!("shaders/opengl/fragment_textured_110.glsl");
const FRAGMENT_SHADER_COLOR: &str = include_str!("shaders/opengl/fragment_color_110.glsl");
const FRAGMENT_SHADER_WATER: &str = include_str!("shaders/opengl/fragment_water_110.glsl");

const VERTEX_SHADER_BASIC_GLES: &str = include_str!("shaders/opengles/vertex_basic_100.glsl");
const FRAGMENT_SHADER_TEXTURED_GLES: &str = include_str!("shaders/opengles/fragment_textured_100.glsl");
const FRAGMENT_SHADER_COLOR_GLES: &str = include_str!("shaders/opengles/fragment_color_100.glsl");

#[derive(Copy, Clone)]
struct RenderShader {
    program_id: GLuint,
    texture: GLint,
    proj_mtx: GLint,
    scale: GLint,
    time: GLint,
    frame_offset: GLint,
    position: GLuint,
    uv: GLuint,
    color: GLuint,
}

impl Default for RenderShader {
    fn default() -> Self {
        Self {
            program_id: 0,
            texture: 0,
            proj_mtx: 0,
            scale: 0,
            time: 0,
            frame_offset: 0,
            position: 0,
            uv: 0,
            color: 0,
        }
    }
}

impl RenderShader {
    fn compile(gl: &Gl, vertex_shader: &str, fragment_shader: &str) -> GameResult<RenderShader> {
        let mut shader = RenderShader::default();
        unsafe {
            shader.program_id = gl.gl.CreateProgram();

            unsafe fn cleanup(shader: &mut RenderShader, gl: &Gl, vert: GLuint, frag: GLuint) {
                if vert != 0 {
                    gl.gl.DeleteShader(vert);
                }

                if frag != 0 {
                    gl.gl.DeleteShader(frag);
                }

                if shader.program_id != 0 {
                    gl.gl.DeleteProgram(shader.program_id);
                    shader.program_id = 0;
                }

                *shader = RenderShader::default();
            }

            let vert_shader = gl.gl.CreateShader(gl::VERTEX_SHADER);
            let frag_shader = gl.gl.CreateShader(gl::FRAGMENT_SHADER);

            let vert_sources = [vertex_shader.as_ptr() as *const GLchar];
            let frag_sources = [fragment_shader.as_ptr() as *const GLchar];
            let vert_sources_len = [vertex_shader.len() as GLint - 1];
            let frag_sources_len = [fragment_shader.len() as GLint - 1];

            gl.gl.ShaderSource(vert_shader, 1, vert_sources.as_ptr(), vert_sources_len.as_ptr());
            gl.gl.ShaderSource(frag_shader, 1, frag_sources.as_ptr(), frag_sources_len.as_ptr());

            gl.gl.CompileShader(vert_shader);
            gl.gl.CompileShader(frag_shader);

            if let Err(e) = check_shader_compile_status(vert_shader, gl) {
                cleanup(&mut shader, gl, vert_shader, frag_shader);
                return Err(e);
            }

            if let Err(e) = check_shader_compile_status(frag_shader, gl) {
                cleanup(&mut shader, gl, vert_shader, frag_shader);
                return Err(e);
            }

            gl.gl.AttachShader(shader.program_id, vert_shader);
            gl.gl.AttachShader(shader.program_id, frag_shader);
            gl.gl.LinkProgram(shader.program_id);

            shader.texture = gl.gl.GetUniformLocation(shader.program_id, b"Texture\0".as_ptr() as _);
            shader.proj_mtx = gl.gl.GetUniformLocation(shader.program_id, b"ProjMtx\0".as_ptr() as _);
            shader.scale = gl.gl.GetUniformLocation(shader.program_id, b"Scale\0".as_ptr() as _) as _;
            shader.time = gl.gl.GetUniformLocation(shader.program_id, b"Time\0".as_ptr() as _) as _;
            shader.frame_offset = gl.gl.GetUniformLocation(shader.program_id, b"FrameOffset\0".as_ptr() as _) as _;
            shader.position = gl.gl.GetAttribLocation(shader.program_id, b"Position\0".as_ptr() as _) as _;
            shader.uv = gl.gl.GetAttribLocation(shader.program_id, b"UV\0".as_ptr() as _) as _;
            shader.color = gl.gl.GetAttribLocation(shader.program_id, b"Color\0".as_ptr() as _) as _;
        }

        Ok(shader)
    }

    unsafe fn bind_attrib_pointer(&self, gl: &Gl, vbo: GLuint) -> GameResult {
        gl.gl.UseProgram(self.program_id);
        gl.gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.gl.EnableVertexAttribArray(self.position);
        gl.gl.EnableVertexAttribArray(self.uv);
        gl.gl.EnableVertexAttribArray(self.color);

        gl.gl.VertexAttribPointer(
            self.position,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<VertexData>() as _,
            field_offset::<VertexData, _, _>(|v| &v.position) as _,
        );

        gl.gl.VertexAttribPointer(
            self.uv,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<VertexData>() as _,
            field_offset::<VertexData, _, _>(|v| &v.uv) as _,
        );

        gl.gl.VertexAttribPointer(
            self.color,
            4,
            gl::UNSIGNED_BYTE,
            gl::TRUE,
            mem::size_of::<VertexData>() as _,
            field_offset::<VertexData, _, _>(|v| &v.color) as _,
        );

        Ok(())
    }
}

struct RenderData {
    initialized: bool,
    tex_shader: RenderShader,
    fill_shader: RenderShader,
    fill_water_shader: RenderShader,
    vbo: GLuint,
    ebo: GLuint,
    font_texture: GLuint,
    font_tex_size: (f32, f32),
    surf_framebuffer: GLuint,
    surf_texture: GLuint,
    last_size: (u32, u32),
}

impl RenderData {
    fn new() -> Self {
        RenderData {
            initialized: false,
            tex_shader: RenderShader::default(),
            fill_shader: RenderShader::default(),
            fill_water_shader: RenderShader::default(),
            vbo: 0,
            ebo: 0,
            font_texture: 0,
            font_tex_size: (1.0, 1.0),
            surf_framebuffer: 0,
            surf_texture: 0,
            last_size: (320, 240),
        }
    }

    fn init(&mut self, gles2_mode: bool, imgui: &mut imgui::Context, gl: &Gl) {
        self.initialized = true;

        let vshdr_basic = if gles2_mode { VERTEX_SHADER_BASIC_GLES } else { VERTEX_SHADER_BASIC };
        let fshdr_tex = if gles2_mode { FRAGMENT_SHADER_TEXTURED_GLES } else { FRAGMENT_SHADER_TEXTURED };
        let fshdr_fill = if gles2_mode { FRAGMENT_SHADER_COLOR_GLES } else { FRAGMENT_SHADER_COLOR };
        let fshdr_fill_water = if gles2_mode { FRAGMENT_SHADER_COLOR_GLES } else { FRAGMENT_SHADER_WATER };

        unsafe {
            self.tex_shader =
                RenderShader::compile(gl, vshdr_basic, fshdr_tex).unwrap_or_else(|_| RenderShader::default());
            self.fill_shader =
                RenderShader::compile(gl, vshdr_basic, fshdr_fill).unwrap_or_else(|_| RenderShader::default());
            self.fill_water_shader =
                RenderShader::compile(gl, vshdr_basic, fshdr_fill_water).unwrap_or_else(|_| RenderShader::default());

            self.vbo = return_param(|x| gl.gl.GenBuffers(1, x));
            self.ebo = return_param(|x| gl.gl.GenBuffers(1, x));

            self.font_texture = return_param(|x| gl.gl.GenTextures(1, x));
            gl.gl.BindTexture(gl::TEXTURE_2D, self.font_texture);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);

            {
                let mut atlas = imgui.fonts();

                let texture = atlas.build_rgba32_texture();
                self.font_tex_size = (texture.width as _, texture.height as _);

                gl.gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    texture.width as _,
                    texture.height as _,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    texture.data.as_ptr() as _,
                );

                atlas.tex_id = (self.font_texture as usize).into();
            }

            let texture_id = return_param(|x| gl.gl.GenTextures(1, x));

            gl.gl.BindTexture(gl::TEXTURE_2D, texture_id);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);

            gl.gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as _,
                320 as _,
                240 as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                null() as _,
            );

            gl.gl.BindTexture(gl::TEXTURE_2D, 0 as _);

            self.surf_texture = texture_id;

            let framebuffer_id = return_param(|x| gl.gl.GenFramebuffers(1, x));

            gl.gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
            gl.gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);
            let draw_buffers = [gl::COLOR_ATTACHMENT0];
            gl.gl.DrawBuffers(1, draw_buffers.as_ptr() as _);

            self.surf_framebuffer = framebuffer_id;
        }
    }
}

pub struct Gl {
    pub gl: gl::Gles2,
}

static mut GL_PROC: Option<Gl> = None;

pub fn load_gl(gl_context: &mut GLContext) -> &'static Gl {
    unsafe {
        if let Some(gl) = &GL_PROC {
            return gl;
        }

        let gl = gl::Gles2::load_with(|ptr| (gl_context.get_proc_address)(&mut gl_context.user_data, ptr));

        let version = {
            let p = gl.GetString(gl::VERSION);
            if p.is_null() {
                "unknown".to_owned()
            } else {
                let data = CStr::from_ptr(p as *const _).to_bytes().to_vec();
                String::from_utf8(data).unwrap()
            }
        };

        log::info!("OpenGL version {}", version);

        GL_PROC = Some(Gl { gl });
        GL_PROC.as_ref().unwrap()
    }
}

struct ImportedModel {
    pub model: Model<PhysicalMaterial>,
    pub time: f32, //incremented on a by-tick basis (+=)
    pub offset_time: f32, //used for between-frame interpolation (=)
    pub stop_time: f32,
    pub play: bool, //true if time should automatically increment with the game's ticks
}

pub struct ThreeDModelSetup {
    vp: Viewport, //screen size
    context: ThreeDContext, //three_d::core::context constructed from "glow" context: three_d::context::Context 
    camera: Camera, //observation location of the 3D meshes
    char_plane: Gm<Mesh, BufferMaterial>, //2d image that holds the user character and interractable elements
    
    char_plane_scale: f32, //scale of the char plane (dynamic scaling of the main d-rs engine)
    frame_xy: (f32, f32), //location of the upper-level game frame (1m = 16 px)
    
    map_models: HashMap<i32, ImportedModel>, //a list of map meshes //TODO: draw all of them
    lights: Vec<Box<dyn Light>>, //list of lights in the model

    skybox: Option<Skybox>, //the skybox, black by default
    ambient_light: AmbientLight, //only one is needed (holds the texture for the skybox)


    midstep_surface: Texture2D,
    midstep_depth: DepthTexture2D,
    midstep_program: Program,

    //clone of the one from RenderData, since we need to know where to put our drawn stuff if we're drawing to the default location
    surf_framebuffer: GLuint,

}

impl ThreeDModelSetup {

    /// Takes a refrence to the d-rs opengl context and the framebuffer ID of the "default" surface it should draw to if a BackendTexture is invalid
    pub fn new(gl_context: &mut GLContext, surf_framebuffer: GLuint) -> ThreeDModelSetup {

        let gl = unsafe{
            // F: FnMut(&'static str) -> *const __gl_imports::raw::c_void,
            // let gl = gl::Gles2::load_with(|ptr| (gl_context.get_proc_address)(&mut gl_context.user_data, ptr));

            // F: FnMut(&str) -> *const std::os::raw::c_void,
            GlowContext::from_loader_function(|ptr| (gl_context.get_proc_address)(&mut gl_context.user_data, ptr) as *const _)
        };
        
        // Get the graphics context from the window
        let context: ThreeDContext = ThreeDContext::from_gl_context(gl.into()).unwrap();
        

        //initial viewport size, should change with screen resizes
        let vp = Viewport {
            x: 0,
            y: 0,
            width: 640,
            height: 480,
        };

        // Create a camera
        let mut camera = Camera::new_perspective(
            vp,
            vec3(0.0, 4.0, 16.0), //(4.0, 1.5, 4.0)
            vec3(0.0, 0.0, 0.0), //(0.0, 1.0, 0.0)
            vec3(0.0, 1.0, 0.0), //(0.0, 1.0, 0.0)
            degrees(45.0),
            0.1,
            1000.0,
        );


        //for mid-copy operation
        let midstep_surface = Texture2D::new_empty::<[u8; 4]>(
            &context,
            vp.width,
            vp.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );

        //for depth probing
        let mut midstep_depth = DepthTexture2D::new::<f32>(
            &context,
            vp.width,
            vp.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge
        );
        

        let midstep_program = Program::from_source(
            &context,
            include_str!("shaders/threed/simple_c_shader.vert"),
            include_str!("shaders/threed/simple_c_shader.frag"),
        );
        if let Err(errr) = &midstep_program {
            log::info!("{}", errr);
        }
        let midstep_program = midstep_program.unwrap();


        //holds the PC and other 2d elements
        //note: big number needed to set the axis-aligned-bounding-box (aabb) will be within frame for the whole map
        //because it is not updated after the mesh is constructed...
        //todo: construct a mesh directly so we can define the aabb ourselves, since it supports an "infinite" size
        let mut plane = Self::new_rectangle(100000.0, 100000.0);
        let mut char_plane: Gm<Mesh, BufferMaterial> = Gm::new(
            Mesh::new(&context, &plane),
            // ColorMaterial{
            //     color: Srgba::new(0, 0, 0, 0),
            //     is_transparent: false,
            //     ..Default::default()
            // }
            BufferMaterial::new(true, 2), //default tex ID is 2 for now...
        );

        // Add an animation to the mesh.
        //char_plane.set_animation(|time| Mat4::from_angle_y(radians(time * 0.005)));

        // //load gltf asset
        // let mut loaded = if let Ok(loaded) =
        // three_d_asset::io::load_async(&["../assets/BoxAnimated.gltf"])
        // {
        //     loaded
        // }

        //empty skybox + ambient light

        //don't make the skybox for now since we need a texture for that
        let ambient_light = AmbientLight::new(&context, 0.5, Srgba::WHITE);

        
        let mut model_list: HashMap<i32, ImportedModel> = HashMap::new();
        let mut lights: Vec<Box<dyn Light>> = Vec::new();

        
        log::info!("light count: {}", lights.len());

        let mut mo = ThreeDModelSetup {
            vp,
            context,
            camera,
            char_plane,
            char_plane_scale: 1.0,
            frame_xy: (0.0,0.0),
            map_models: model_list,
            lights,
            surf_framebuffer,

            skybox: None,
            ambient_light,

            midstep_surface,
            midstep_depth,
            midstep_program,
        };


        /* 
        //test: load object 1
        {
            let assets = three_d_asset::io::load(&["C:/Users/EdwardStuckey/Documents/GitHub/CaveS-Public/Dim3/meshes/testOrigin.glb"]);
        
            
            if let Ok(mut raw_assets) = assets {
    
                let mut pathh = path::PathBuf::new();
                pathh.push("testOrigin.glb");

                let assets = raw_assets.get(pathh).unwrap();

                let result = mo.load_gltf(assets, 0, true);

                if result.is_err() {
                    let mut pear = 5 + 2;
                    let mut aa = pear + 2;

                }

                // let (mm, mut light_list) = deserialize_gltf(&context, &mut raw_assets, &pathh).unwrap();
                // let mut cpu_model: CpuModel = mm.into();
    
                // //let mut cpu_model: CpuModel = raw_assets.deserialize("testOrigin.glb").unwrap();
                // cpu_model
                //     .geometries
                //     .iter_mut()
                //     .for_each(|part| part.compute_normals());
    
                // let mut model = Model::<PhysicalMaterial>::new(&context, &cpu_model).unwrap();
    
                // lights = light_list;
    
                // model_list.insert(0, model);
            }
        }

        //test: load object 2
        {
            let assets = three_d_asset::io::load(&["C:/Users/EdwardStuckey/Documents/GitHub/CaveS-Public/Dim3/meshes/testOrigin_box.glb"]);
        
            if let Ok(mut raw_assets) = assets {
    
                let mut pathh = path::PathBuf::new();
                pathh.push("testOrigin_box.glb");

                let assets = raw_assets.get(pathh).unwrap();

                let _ = mo.load_gltf(assets, 1, false);

                // let (mm, _) = deserialize_gltf(&context, &mut raw_assets, &pathh).unwrap();
                // let mut cpu_model: CpuModel = mm.into();
    
                // //let mut cpu_model: CpuModel = raw_assets.deserialize("testOrigin.glb").unwrap();
                // cpu_model
                //     .geometries
                //     .iter_mut()
                //     .for_each(|part| part.compute_normals());
    
                // let mut model = Model::<PhysicalMaterial>::new(&context, &cpu_model).unwrap();

    
                // model_list.insert(1, model);
            }
        }
        */



        mo


                
    }

    /// Loads a GLTF into the three-d backend to be rendered onscreen, returns "true" if the value was inserted, "false" if updated
    pub fn load_gltf(&mut self, data: &[u8], key: i32, update_lights: bool) -> GameResult<bool> {

        //used to store and get the data from the RawAssets container, which the gltf infrastructure uses
        //since raw_assets is no longer used after the deserialize operation, this pathname doesn't really matter
        let path = "FILE";

        //push the file into the raw_assets object
        let mut raw_assets = RawAssets::new();
        raw_assets.insert(path, data.to_vec());


        let mut path_buffer = path::PathBuf::new();
        path_buffer.push(path);
        let (parsed_scene, mut light_list) = deserialize_gltf(&self.context, &mut raw_assets, &path_buffer).unwrap();
        let mut cpu_model: CpuModel = parsed_scene.into();

        //we may or may not need to recompute normals...
        // cpu_model
        //     .geometries
        //     .iter_mut()
        //     .for_each(|part| part.compute_normals());

        let mut model = Model::<PhysicalMaterial>::new(&self.context, &cpu_model).unwrap();

        let im_model = ImportedModel{
            model,
            time: 1.0,
            offset_time: 0.0,
            stop_time: -1.0, //<= 0 time means this setting is inactive
            play: false,
        };

        let result = self.map_models.insert(key, im_model);

        //delete old lights and add new ones
        if update_lights {
            self.lights = light_list;
        }

        Ok(result.is_none())

    }

    /// unloads the GLTF at the provided index, returns error if OOB
    pub fn unload_gltf(&mut self, key: i32) -> GameResult<bool> {

        let old_item = self.map_models.remove(&key);        
        Ok(old_item.is_some())
    
    }

    /// unloads all GLTF objects currently in the hash map
    pub fn clear_gltf(&mut self) {
        self.map_models.clear();
    }

    /// load the skybox and optionally apply its texture to the ambient light
    pub fn load_skybox(&mut self, data: &[u8], have_ambient: bool) -> GameResult {

        //used to store and get the data from the RawAssets container, which the gltf infrastructure uses
        //since raw_assets is no longer used after the deserialize operation, this pathname doesn't really matter
        let path = "FILE";

        //push the file into the raw_assets object
        let mut raw_assets = RawAssets::new();
        raw_assets.insert(path, data.to_vec());


        let mut path_buffer = path::PathBuf::new();
        path_buffer.push(path);
        let image = raw_assets.deserialize(path).unwrap();

        self.skybox = Some(Skybox::new_from_equirectangular(
            &self.context,
            &image,
        ));

        //update ambient texture if required
        if have_ambient {
            if let Some(skybox) = &mut self.skybox {
                self.ambient_light.environment = Some(Environment::new(&self.context, skybox.texture()));
            }
        }

        Ok(())

    }

    /// Nullifies skybox
    pub fn unload_skybox(&mut self) {
        self.skybox = None;
    }

    /// Sets various attribues of the ambient light
    pub fn set_ambient_attributes(&mut self, data: Option<&[u8]>, color: Option<Color>, intensity: Option<f32>) -> GameResult {

        //load new reflection texture
        if let Some(data) = data {
            let path = "FILE";

            //push the file into the raw_assets object
            let mut raw_assets = RawAssets::new();
            raw_assets.insert(path, data.to_vec());
    
    
            let mut path_buffer = path::PathBuf::new();
            path_buffer.push(path);
            let mut image: CpuTexture = raw_assets.deserialize(path).unwrap();

            let cm = match image.data {
                TextureData::RgbaU8(_) | TextureData::RgbU8(_) => {
                    let mut cpu_texture = image.clone();
                    &image.data.to_linear_srgb();
                    TextureCubeMap::new_from_equirectangular::<u8>(&self.context, &image)
                }
                TextureData::RgU8(_) | TextureData::RU8(_) => {
                    TextureCubeMap::new_from_equirectangular::<u8>(&self.context, &image)
                }
                TextureData::RgbaF16(_)
                | TextureData::RgbF16(_)
                | TextureData::RgF16(_)
                | TextureData::RF16(_) => {
                    TextureCubeMap::new_from_equirectangular::<f16>(&self.context, &image)
                }
                TextureData::RgbaF32(_)
                | TextureData::RgbF32(_)
                | TextureData::RgF32(_)
                | TextureData::RF32(_) => {
                    TextureCubeMap::new_from_equirectangular::<f32>(&self.context, &image)
                }
            };

            //let cm = TextureCubeMap::new_from_equirectangular(&self.context, &image);
            self.ambient_light.environment = Some(Environment::new(&self.context, &cm));

        }

        if let Some(color) = color {
            let uu = color.to_rgba();
            let compat_color = Srgba::new(uu.0, uu.1, uu.2, uu.3);
            self.ambient_light.color = compat_color;
        }

        if let Some(intensity) = intensity {
            self.ambient_light.intensity = intensity;
        }

        Ok(())

    }

    /// Unloads the cubemap from the ambient light
    pub fn unload_ambient_image(&mut self) {
        self.ambient_light.environment = None;
    }

    /// Tells the 3D context to use use this surface to draw onto the char plane.
    pub fn set_char_plane_target_surf(&mut self, texture: &Box<dyn BackendTexture>) -> GameResult {
        let gl_texture = texture
            .as_any()
            .downcast_ref::<OpenGLTexture>()
            .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))?;

        self.set_char_plane_target_no(gl_texture.texture_id)

    }

    /// same as `set_char_plane_target_surf` except it takes a raw openGL surface ID
    pub fn set_char_plane_target_no(&mut self, target: u32) -> GameResult {

        self.narc();

        if let Some(num) = NonZeroU32::new(target) {
            self.char_plane.material.tex_id = num;
            Ok(())
        } else {
            Err(GameError::InvalidValue(format!("Number {} was not a valid non-zero u32", target)))
        }

    }

    /// returns the number currently being used as the texture target
    pub fn get_char_plane_target(&self) -> u32 {
        let num = self.char_plane.material.tex_id.get();
        num
    }

    /// check for openGL backend errors
    pub fn narc(&mut self) {
        let resultt = self.context.error_check();
        if let Err(prob) = resultt {
            log::info!("ERR: {}", prob);
        }
    }

    /// draws to this texture. If null, it draws directly to the screenbuffer instead
    pub fn draw(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        if let Some(texture) = texture {
            let gl_texture = texture
                .as_any()
                .downcast_ref::<OpenGLTexture>()
                .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))?;

            //draw to opengl framebuffer
            self.draw_no(gl_texture.framebuffer_id)?;

        } else {
            //draw to the shared screenbuffer
            self.draw_no(self.surf_framebuffer)?;
        }

        Ok(())
    }

    /// draws all meshes and the PC plane to the gl surface set by the passed-in argument
    /// Note that due to the framework, running this resets the binding back to 0 when finished
    pub fn draw_no(&mut self, dest_id: u32) -> GameResult {

        //update char plane, camera, and animations
        self.displace_char_plane();
        self.displace_camera();
        self.run_animations();

        self.narc();

        //what texture we're targeting (if 0, target the screenbuffer. Otherwise, target the ID we've been given)
        let mut render_target = //ManuallyDrop::new(
        if dest_id == 0 {
            RenderTarget::screen(&self.context, self.vp.width, self.vp.height)
        } else {
            let destination = NonZeroU32::new(dest_id);
            if (destination.is_none()) {
                return Err(GameError::InvalidValue(format!("Number {} was not a valid non-zero u32", dest_id)));
            }
            let destination = destination.unwrap();
            self.narc();
            let fb = NativeFramebuffer(destination);
            self.narc();
            RenderTarget::from_framebuffer(&self.context, self.vp.width, self.vp.height, fb)
        };
        //);

        self.narc();
        

        //draw to midsurface
        {
            //no need to clear: the normal stuff already does this.
            //will need to clear if we're putting this on a seaprate surface...
            // unsafe {
            //     model.context.clear_color(0.0, 0.0, 0.0, 1.0);
            //     model.context.clear(context::COLOR_BUFFER_BIT | context::DEPTH_BUFFER_BIT);
            //     //context.bind_buffer(target, buffer);
            //     //context.set_blend(blend);
            //     //context.bind_framebuffer(context::FRAMEBUFFER, Some(32));
            // }

            //conglomerate all models into an iterator to be rendered by the render target
            // let mut renderable_things: Box<dyn Iterator<Item = _>> = Box::new(std::iter::empty());
            // for (key, model) in &self.map_models {
            //     renderable_things = Box::new(renderable_things.chain(model.into_iter()));
            // }


            let renderable_things = self.map_models.iter().fold(
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>,
                |acc, (key, model)| Box::new(acc.chain(model.model.into_iter()))
            );
            //note: I'm not sure if using boxes here is better than calling '.render' multiple times in quick succession, but it works, so I'll take it.


            let one_iter: std::iter::Once<&dyn Light> = std::iter::once(&self.ambient_light);
            let lightable_things  = self.lights.iter().map(|l| l.as_ref()).chain(one_iter).collect::<Vec<_>>(); //.into_iter().chain(self.ambient_light.into()).collect::<Vec<_>>();

            //render to mid-surface
            RenderTarget::new(
                self.midstep_surface.as_color_target(None),
                self.midstep_depth.as_depth_target())
                .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
                .render(&self.camera, &self.skybox, &[]) //todo: add this to main "renderable things"
                .render(
                    &self.camera,
                    renderable_things, //self.map_models.into_iter(), //iter().map(|x| x as &dyn Object),
                    &lightable_things) //&self.lights.iter().map(|l| l.as_ref()).collect::<Vec<_>>())
                .render(&self.camera, &self.char_plane, &[]);
                


        }

        //copy midsurface to output surface
        unsafe {
            let fbb = render_target.into_framebuffer();
            //self.context.clear_color(0.0, 0.0, 0.0, 1.0);
            //self.context.clear(context::COLOR_BUFFER_BIT | context::DEPTH_BUFFER_BIT);
            //self.context.bind_buffer(target, buffer);

            self.context.set_blend(Blend::TRANSPARENCY);
            self.context.bind_framebuffer(context::FRAMEBUFFER, fbb);

            let positions = VertexBuffer::new_with_data(
                &self.context,
                &[
                    vec2(-1.0, -1.0), 
                    vec2(-1.0, 1.0),
                    vec2(1.0, 1.0),
                    
                    vec2(-1.0, -1.0), 
                    vec2(1.0, 1.0),
                    vec2(1.0, -1.0),
                ],
            );
            let uvs = VertexBuffer::new_with_data(
                &self.context,
                &[
                    vec2(0.0, 0.0), 
                    vec2(0.0, 1.0),
                    vec2(1.0, 1.0),

                    vec2(0.0, 0.0),
                    vec2(1.0, 1.0),
                    vec2(1.0, 0.0),
                ],
            );
            let colors = VertexBuffer::new_with_data(
                &self.context,
                &[
                    Srgba::WHITE.to_linear_srgb(),
                    Srgba::WHITE.to_linear_srgb(),
                    Srgba::WHITE.to_linear_srgb(),
                    Srgba::WHITE.to_linear_srgb(),
                    Srgba::WHITE.to_linear_srgb(),
                    Srgba::WHITE.to_linear_srgb(),
                ],
            );

            let transform = Mat3::identity();
            self.midstep_program.use_uniform("textureTransformation", transform);
            self.midstep_program.use_texture("tex", &self.midstep_surface);

            self.midstep_program.use_vertex_attribute("position", &positions);
            self.midstep_program.use_vertex_attribute("uv", &uvs);
            self.midstep_program.use_vertex_attribute("color", &colors);
            self.midstep_program.draw_arrays(
                RenderStates::default(),
                self.vp,
                positions.vertex_count(),
            );


            



            //self.midstep_program.draw_elements(render_states, viewport, element_buffer);

        }      

        self.narc();

        Ok(())


    }

    /// Set width and height of the viewport, char plane, and intermediate surfaces
    pub fn set_viewport_size(&mut self, width: u32, height: u32, scale: f32) {
        
        self.char_plane_scale = scale;
        
        self.vp.width = width;
        self.vp.height = height;


        //recreate midstep surface with new size
        self.midstep_surface = Texture2D::new_empty::<[u8; 4]>(
            &self.context,
            self.vp.width,
            self.vp.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        //ditto with depth probe
        self.midstep_depth = DepthTexture2D::new::<f32>(
            &self.context,
            self.vp.width,
            self.vp.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge
        );
    }

    /// Given an XY coordinate set in meters, set the location of the camera and char plane (origin is the center of the view)
    pub fn set_location(&mut self, x: f32, y: f32) {
        self.frame_xy = (x, y);
    }

    /// Increment the internal animation timers by this ammount (in "seconds", relative to the exported GLTF)
    pub fn increment_animation_time(&mut self, delta_time: f32, offset_time: f32) {


        for (key, model) in &mut self.map_models {
            if(model.play) {

                model.time += delta_time;
                model.offset_time = offset_time;

                //check for animation stopping
                if model.stop_time > 0.0 && model.time >= model.stop_time {
                    model.play = false;
                    model.time = model.stop_time;
                    model.stop_time = -1.0;
                }
            }
        }

    }

    /// Set the animation timer of the model to this value, returns true if the model at `key` exists
    pub fn set_model_anim_time(&mut self, key: i32, time: f32) -> GameResult<bool> {

        if let Some(model) = self.map_models.get_mut(&key) {
            model.time = time;
            return Ok(true);
        }

        Ok(false)
    }

    /// Set the animation stop time for the model, so it will run to this time and pause (numbers < 0 disable stopping)
    pub fn set_model_anim_stop_time(&mut self, key: i32, stop_time: f32) -> GameResult<bool> {

        if let Some(model) = self.map_models.get_mut(&key) {
            model.stop_time = stop_time;
            return Ok(true);
        }

        Ok(false)
    }

    /// Set the animation state of the model to T/F. If true, the model will animate with the game speed
    pub fn set_model_anim_state(&mut self, key: i32, play: bool) -> GameResult<bool> {

        if let Some(model) = self.map_models.get_mut(&key) {
            model.play = play;
            return Ok(true);
        }

        Ok(false)
    }

    /// Set what animation the model should run (these are named ahead of time in Blender)
    pub fn set_model_animation(&mut self, key: i32, anim_name: &str) -> GameResult<bool> {

        if let Some(model) = self.map_models.get_mut(&key) {
            model.model.choose_animation(Some(anim_name));

            return Ok(true);
        }

        Ok(false)

    }


    /// make a rectangle CpuMesh
    fn new_rectangle(width: f32, height: f32) -> CpuMesh {
        //how the points should be indexed
        let indices = vec![0u8, 1, 2, 2, 3, 0];

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let positions = vec![
            Vec3::new(-half_width, -half_height, 0.0),
            Vec3::new(half_width, -half_height, 0.0),
            Vec3::new(half_width, half_height, 0.0),
            Vec3::new(-half_width, half_height, 0.0),
        ];
        let normals = vec![
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let tangents = vec![
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
        ];

        //flip these since d-rs draws stuff upside down
        // let uvs = vec![
        //     Vec2::new(0.0, 1.0),
        //     Vec2::new(1.0, 1.0),
        //     Vec2::new(1.0, 0.0),
        //     Vec2::new(0.0, 0.0),
        // ];
        let uvs = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];

        CpuMesh {
            indices: Indices::U8(indices),
            positions: Positions::F32(positions),
            normals: Some(normals),
            tangents: Some(tangents),
            uvs: Some(uvs),
            ..Default::default()
        }
    }

    /// Move + Resize the internal `char_plane` rectangle to have the correct width and height, centered at `frame_xy` 
    fn displace_char_plane(&mut self) {


        let width = (self.vp.width as f32) / (16.0 * self.char_plane_scale);
        let height = (self.vp.height as f32) / (16.0 * self.char_plane_scale);

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let offx = self.frame_xy.0 + half_width;
        let offy = self.frame_xy.1 - half_height;


        let positions = vec![
            Vec3::new(-half_width + offx, -half_height + offy, 0.0),
            Vec3::new(half_width + offx, -half_height + offy, 0.0),
            Vec3::new(half_width + offx, half_height + offy, 0.0),
            Vec3::new(-half_width + offx, half_height + offy, 0.0),
        ];

        self.char_plane.update_positions(&positions);
    }

    /// Move + resize the camera to match `self.vp`, `frame_xy`, and  `char_plane_wh`
    fn displace_camera(&mut self) {

        let plwidth = (self.vp.width as f32) / (16.0 * self.char_plane_scale);
        let plheight = (self.vp.height as f32) / (16.0 * self.char_plane_scale);
        let half_width = plwidth / 2.0;
        let half_height = plheight / 2.0;


        //log::info!("{} x {}, scale: {}", width, height, scale);

        // Ensure the viewport matches the current window viewport which changes if the window is resized
        self.camera.set_viewport(self.vp);

        let fov = match self.camera.projection_type() {
            ProjectionType::Perspective { field_of_view_y } => {
                field_of_view_y.0
            }
            _ => {
                (45.0 as f32).to_radians() //by default, assume 45 degrees (but this shouldn't be reachable...) (should I put an unreachable panic?)
            }
        };

        let screen_height = (self.vp.height as f32) / self.char_plane_scale;
        //triangulate distance
        let a = (screen_height) / 2.0;
        let b = a / (fov / 2.0).tan();


        let mut pos = self.camera.position().clone();
        pos.z = b / 16.0;
        pos.x = self.frame_xy.0 + half_width;
        pos.y = self.frame_xy.1 - half_height;
        // pos.x = 0.0;
        // pos.y = 0.0;

        let mut tgt = self.camera.target().clone();
        tgt.x = self.frame_xy.0 + half_width;
        tgt.y = self.frame_xy.1 - half_height;
        tgt.z = 0.0;

        let up = self.camera.up().clone().clone();
        self.camera.set_view(pos, tgt, up);


    }

    /// Run animations for all loaded models, even if the model doesn't have an animation
    fn run_animations(&mut self) {

        for (key, model) in &mut self.map_models {
            model.model.animate(model.time + model.offset_time);
        }
    }



}

pub struct OpenGLRenderer {
    refs: GLContext,
    imgui: UnsafeCell<imgui::Context>,
    render_data: RenderData,
    context_active: Arc<RefCell<bool>>,
    def_matrix: [[f32; 4]; 4],
    curr_matrix: [[f32; 4]; 4],
    pub model: Option<ThreeDModelSetup>,
    
}

impl OpenGLRenderer {
    pub fn new(refs: GLContext, imgui: UnsafeCell<imgui::Context>) -> OpenGLRenderer {
        OpenGLRenderer {
            refs,
            imgui,
            render_data: RenderData::new(),
            context_active: Arc::new(RefCell::new(true)),
            def_matrix: [[0.0; 4]; 4],
            curr_matrix: [[0.0; 4]; 4],
            model: None,
        }
    }

    fn get_context(&mut self) -> Option<(&mut GLContext, &'static Gl)> {
        let imgui = unsafe { &mut *self.imgui.get() };

        let gles2 = self.refs.gles2_mode;
        let gl = load_gl(&mut self.refs);

        if !self.render_data.initialized {
            self.render_data.init(gles2, imgui, gl);
        }

        //make new 3D context
        if(self.model.is_none()) {
            self.model = Some(ThreeDModelSetup::new(&mut self.refs, self.render_data.surf_framebuffer));
        }

        Some((&mut self.refs, gl))
    }


}

impl BackendRenderer for OpenGLRenderer {
    fn renderer_name(&self) -> String {
        if self.refs.gles2_mode {
            "OpenGL ES 2.0".to_string()
        } else {
            "OpenGL 2.1".to_string()
        }
    }

    fn clear(&mut self, color: Color) {
        if let Some((_, gl)) = self.get_context() {
            unsafe {
                gl.gl.ClearColor(color.r, color.g, color.b, color.a);
                gl.gl.Clear(gl::COLOR_BUFFER_BIT);
            }
        }
    }

    fn present(&mut self) -> GameResult {
        {
            let mutex = GAME_SUSPENDED.lock().unwrap();
            if *mutex {
                return Ok(());
            }
        }

        unsafe {



            
            if let Some((_, gl)) = self.get_context() {

                handle_err(gl, 0);

                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
                gl.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                let matrix: [[f32; 4]; 4] =
                    [[2.0f32, 0.0, 0.0, 0.0], [0.0, -2.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [-1.0, 1.0, 0.0, 1.0]];

                self.render_data.tex_shader.bind_attrib_pointer(gl, self.render_data.vbo);
                gl.gl.UniformMatrix4fv(self.render_data.tex_shader.proj_mtx, 1, gl::FALSE, matrix.as_ptr() as _);

                let color = (255, 255, 255, 255);
                let vertices = [
                    VertexData { position: (0.0, 1.0), uv: (0.0, 0.0), color },
                    VertexData { position: (0.0, 0.0), uv: (0.0, 1.0), color },
                    VertexData { position: (1.0, 0.0), uv: (1.0, 1.0), color },
                    VertexData { position: (0.0, 1.0), uv: (0.0, 0.0), color },
                    VertexData { position: (1.0, 0.0), uv: (1.0, 1.0), color },
                    VertexData { position: (1.0, 1.0), uv: (1.0, 0.0), color },
                ];

                self.draw_arrays_tex_id(
                    gl::TRIANGLES,
                    &vertices,
                    self.render_data.surf_texture,
                    BackendShader::Texture,
                )?;

                handle_err(gl, 0);

                gl.gl.Finish();

                handle_err(gl, 0);
            }


            // //splice in three-d for testing
            // if let Some(model) = &mut self.model {
            //     //model.set_char_plane_target_no(2);
            //     let result = model.draw_no(0);//(self.render_data.surf_framebuffer);
            //     if(result.is_err()) {
            //         log::info!("ERROR");
            //     }
            // }

            if let Some((context, _)) = self.get_context() {
                (context.swap_buffers)(&mut context.user_data);
            }

        }

        Ok(())
    }

    fn set_vsync_mode(&mut self, mode: VSyncMode) -> GameResult {
        if !self.refs.is_sdl {
            return Ok(());
        }

        #[cfg(feature = "backend-sdl")]
            unsafe {
            let ctx = &mut *self.refs.ctx;

            match mode {
                VSyncMode::Uncapped => {
                    sdl2_sys::SDL_GL_SetSwapInterval(0);
                }
                VSyncMode::VSync => {
                    sdl2_sys::SDL_GL_SetSwapInterval(1);
                }
                _ => {
                    if sdl2_sys::SDL_GL_SetSwapInterval(-1) == -1 {
                        log::warn!("Failed to enable variable refresh rate, falling back to non-V-Sync.");
                        sdl2_sys::SDL_GL_SetSwapInterval(0);
                    }
                }
            }
        }

        Ok(())
    }

    fn prepare_draw(&mut self, width: f32, height: f32) -> GameResult {
        if let Some((_, gl)) = self.get_context() {
            handle_err(gl, 0);
            unsafe {
                let (width_u, height_u) = (width as u32, height as u32);
                if self.render_data.last_size != (width_u, height_u) {
                    self.render_data.last_size = (width_u, height_u);
                    gl.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
                    gl.gl.BindTexture(gl::TEXTURE_2D, self.render_data.surf_texture);

                    gl.gl.TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA as _,
                        width_u as _,
                        height_u as _,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        null() as _,
                    );

                    gl.gl.BindTexture(gl::TEXTURE_2D, 0 as _);
                }

                handle_err(gl, 0);

                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, self.render_data.surf_framebuffer); //aye... this be the problem
                handle_err(gl, 0);

                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, self.render_data.surf_framebuffer);
                handle_err(gl, 0);

                gl.gl.ClearColor(0.0, 0.0, 0.0, 0.0);
                gl.gl.Clear(gl::COLOR_BUFFER_BIT);

                handle_err(gl, 0);
                
                gl.gl.ActiveTexture(gl::TEXTURE0);
                gl.gl.BlendEquation(gl::FUNC_ADD);
                gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                gl.gl.Viewport(0, 0, width_u as _, height_u as _);

                handle_err(gl, 0);

                self.def_matrix = [
                    [2.0 / width, 0.0, 0.0, 0.0],
                    [0.0, 2.0 / -height, 0.0, 0.0],
                    [0.0, 0.0, -1.0, 0.0],
                    [-1.0, 1.0, 0.0, 1.0],
                ];
                self.curr_matrix = self.def_matrix;

                handle_err(gl, 0);

                gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
                gl.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                gl.gl.UseProgram(self.render_data.fill_shader.program_id);
                gl.gl.UniformMatrix4fv(
                    self.render_data.fill_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );
                gl.gl.UseProgram(self.render_data.fill_water_shader.program_id);
                gl.gl.Uniform1i(self.render_data.fill_water_shader.texture, 0);
                gl.gl.UniformMatrix4fv(
                    self.render_data.fill_water_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );
                gl.gl.UseProgram(self.render_data.tex_shader.program_id);
                gl.gl.Uniform1i(self.render_data.tex_shader.texture, 0);
                gl.gl.UniformMatrix4fv(
                    self.render_data.tex_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );
                handle_err(gl, 0);
            }

            handle_err(gl, 0);
            Ok(())
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        if let Some((_, gl)) = self.get_context() {
            handle_err(gl, 0);
            unsafe {
                let current_texture_id = return_param(|x| gl.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, x)) as u32;
                let texture_id = return_param(|x| gl.gl.GenTextures(1, x));

                gl.gl.BindTexture(gl::TEXTURE_2D, texture_id);
                gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

                gl.gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    width as _,
                    height as _,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    null() as _,
                );

                gl.gl.BindTexture(gl::TEXTURE_2D, current_texture_id);

                let framebuffer_id = return_param(|x| gl.gl.GenFramebuffers(1, x));

                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
                gl.gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);
                let draw_buffers = [gl::COLOR_ATTACHMENT0];
                gl.gl.DrawBuffers(1, draw_buffers.as_ptr() as _);

                gl.gl.Viewport(0, 0, width as _, height as _);
                gl.gl.ClearColor(0.0, 0.0, 0.0, 0.0);
                gl.gl.Clear(gl::COLOR_BUFFER_BIT);
                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);

                // todo error checking: glCheckFramebufferStatus()
                handle_err(gl, 0);

                Ok(Box::new(OpenGLTexture {
                    texture_id,
                    framebuffer_id,
                    width,
                    height,
                    vertices: Vec::new(),
                    shader: self.render_data.tex_shader,
                    vbo: self.render_data.vbo,
                    context_active: self.context_active.clone(),
                }))
            }
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        if let Some((_, gl)) = self.get_context() {
            handle_err(gl, 0);
            unsafe {
                let current_texture_id = return_param(|x| gl.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, x)) as u32;
                let texture_id = return_param(|x| gl.gl.GenTextures(1, x));
                gl.gl.BindTexture(gl::TEXTURE_2D, texture_id);
                gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

                gl.gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    width as _,
                    height as _,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr() as _,
                );

                gl.gl.BindTexture(gl::TEXTURE_2D, current_texture_id);

                handle_err(gl, 0);
                Ok(Box::new(OpenGLTexture {
                    texture_id,
                    framebuffer_id: 0,
                    width,
                    height,
                    vertices: Vec::new(),
                    shader: self.render_data.tex_shader,
                    vbo: self.render_data.vbo,
                    context_active: self.context_active.clone(),
                }))
            }
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult {
        if let Some((_, gl)) = self.get_context() {
            handle_err(gl, 0);
            match blend {
                BlendMode::Add => unsafe {
                    gl.gl.Enable(gl::BLEND);
                    gl.gl.BlendEquation(gl::FUNC_ADD);
                    gl.gl.BlendFunc(gl::ONE, gl::ONE);
                },
                BlendMode::Alpha => unsafe {
                    gl.gl.Enable(gl::BLEND);
                    gl.gl.BlendEquation(gl::FUNC_ADD);
                    gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                },
                BlendMode::Multiply => unsafe {
                    gl.gl.Enable(gl::BLEND);
                    gl.gl.BlendEquation(gl::FUNC_ADD);
                    gl.gl.BlendFuncSeparate(gl::ZERO, gl::SRC_COLOR, gl::ZERO, gl::SRC_ALPHA);
                },
                BlendMode::None => unsafe {
                    gl.gl.Disable(gl::BLEND);
                },
            }
            handle_err(gl, 0);
            Ok(())
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        if let Some((_, gl)) = self.get_context() {
            handle_err(gl, 0);
            unsafe {
                if let Some(texture) = texture {
                    let gl_texture = texture
                        .as_any()
                        .downcast_ref::<OpenGLTexture>()
                        .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))?;

                    self.curr_matrix = [
                        [2.0 / (gl_texture.width as f32), 0.0, 0.0, 0.0],
                        [0.0, 2.0 / (gl_texture.height as f32), 0.0, 0.0],
                        [0.0, 0.0, -1.0, 0.0],
                        [-1.0, -1.0, 0.0, 1.0],
                    ];

                    gl.gl.UseProgram(self.render_data.fill_shader.program_id);
                    gl.gl.UniformMatrix4fv(
                        self.render_data.fill_shader.proj_mtx,
                        1,
                        gl::FALSE,
                        self.curr_matrix.as_ptr() as _,
                    );
                    gl.gl.UseProgram(self.render_data.fill_water_shader.program_id);
                    gl.gl.UniformMatrix4fv(
                        self.render_data.fill_water_shader.proj_mtx,
                        1,
                        gl::FALSE,
                        self.curr_matrix.as_ptr() as _,
                    );
                    gl.gl.UseProgram(self.render_data.tex_shader.program_id);
                    gl.gl.Uniform1i(self.render_data.tex_shader.texture, 0);
                    gl.gl.UniformMatrix4fv(
                        self.render_data.tex_shader.proj_mtx,
                        1,
                        gl::FALSE,
                        self.curr_matrix.as_ptr() as _,
                    );

                    gl.gl.BindFramebuffer(gl::FRAMEBUFFER, gl_texture.framebuffer_id);
                    gl.gl.Viewport(0, 0, gl_texture.width as _, gl_texture.height as _);
                } else {
                    self.curr_matrix = self.def_matrix;

                    gl.gl.UseProgram(self.render_data.fill_shader.program_id);
                    gl.gl.UniformMatrix4fv(
                        self.render_data.fill_shader.proj_mtx,
                        1,
                        gl::FALSE,
                        self.curr_matrix.as_ptr() as _,
                    );
                    gl.gl.UseProgram(self.render_data.fill_water_shader.program_id);
                    gl.gl.UniformMatrix4fv(
                        self.render_data.fill_water_shader.proj_mtx,
                        1,
                        gl::FALSE,
                        self.curr_matrix.as_ptr() as _,
                    );
                    gl.gl.UseProgram(self.render_data.tex_shader.program_id);
                    gl.gl.Uniform1i(self.render_data.tex_shader.texture, 0);
                    gl.gl.UniformMatrix4fv(
                        self.render_data.tex_shader.proj_mtx,
                        1,
                        gl::FALSE,
                        self.curr_matrix.as_ptr() as _,
                    );
                    gl.gl.BindFramebuffer(gl::FRAMEBUFFER, self.render_data.surf_framebuffer);
                    gl.gl.Viewport(0, 0, self.render_data.last_size.0 as _, self.render_data.last_size.1 as _);
                }
            }
            handle_err(gl, 0);
            Ok(())
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult {
        unsafe {
            if let Some(gl) = &GL_PROC {
                handle_err(gl, 0);
                let color = color.to_rgba();
                let mut uv = self.render_data.font_tex_size;
                uv.0 = 0.0 / uv.0;
                uv.1 = 0.0 / uv.1;

                let vertices = [
                    VertexData { position: (rect.left as _, rect.bottom as _), uv, color },
                    VertexData { position: (rect.left as _, rect.top as _), uv, color },
                    VertexData { position: (rect.right as _, rect.top as _), uv, color },
                    VertexData { position: (rect.left as _, rect.bottom as _), uv, color },
                    VertexData { position: (rect.right as _, rect.top as _), uv, color },
                    VertexData { position: (rect.right as _, rect.bottom as _), uv, color },
                ];

                self.render_data.fill_shader.bind_attrib_pointer(gl, self.render_data.vbo);

                gl.gl.BindTexture(gl::TEXTURE_2D, self.render_data.font_texture);
                gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.render_data.vbo);
                gl.gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices.len() * mem::size_of::<VertexData>()) as _,
                    vertices.as_ptr() as _,
                    gl::STREAM_DRAW,
                );

                gl.gl.DrawArrays(gl::TRIANGLES, 0, vertices.len() as _);

                gl.gl.BindTexture(gl::TEXTURE_2D, 0);
                gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
                handle_err(gl, 0);
                Ok(())
            } else {
                Err(RenderError("No OpenGL context available!".to_string()))
            }
        }
    }

    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult {
        if let Some((_, gl)) = self.get_context() {
            handle_err(gl, 0);
            unsafe {
                if let Some(rect) = &rect {
                    gl.gl.Enable(gl::SCISSOR_TEST);
                    gl.gl.Scissor(
                        rect.left as GLint,
                        self.render_data.last_size.1 as GLint - rect.bottom as GLint,
                        rect.width() as GLint,
                        rect.height() as GLint,
                    );
                } else {
                    gl.gl.Disable(gl::SCISSOR_TEST);
                }
            }
            handle_err(gl, 0);

            Ok(())
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.imgui.get()) }
    }

    fn imgui_texture_id(&self, texture: &Box<dyn BackendTexture>) -> GameResult<TextureId> {
        let gl_texture = texture
            .as_any()
            .downcast_ref::<OpenGLTexture>()
            .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))?;

        Ok(TextureId::new(gl_texture.texture_id as usize))
    }

    fn prepare_imgui(&mut self, _ui: &Ui) -> GameResult {
        Ok(())
    }

    fn render_imgui(&mut self, draw_data: &DrawData) -> GameResult {
        // https://github.com/michaelfairley/rust-imgui-opengl-renderer
        if let Some((_, gl)) = self.get_context() {
            unsafe {
                gl.gl.ActiveTexture(gl::TEXTURE0);
                gl.gl.Enable(gl::BLEND);
                gl.gl.BlendEquation(gl::FUNC_ADD);
                gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl.gl.Disable(gl::CULL_FACE);
                gl.gl.Disable(gl::DEPTH_TEST);
                gl.gl.Enable(gl::SCISSOR_TEST);

                let imgui = self.imgui()?;
                let [width, height] = imgui.io().display_size;
                let [scale_w, scale_h] = imgui.io().display_framebuffer_scale;

                let fb_width = width * scale_w;
                let fb_height = height * scale_h;

                gl.gl.Viewport(0, 0, fb_width as _, fb_height as _);
                let matrix = [
                    [2.0 / width as f32, 0.0, 0.0, 0.0],
                    [0.0, 2.0 / -(height as f32), 0.0, 0.0],
                    [0.0, 0.0, -1.0, 0.0],
                    [-1.0, 1.0, 0.0, 1.0],
                ];

                gl.gl.UseProgram(self.render_data.tex_shader.program_id);
                gl.gl.Uniform1i(self.render_data.tex_shader.texture, 0);
                gl.gl.UniformMatrix4fv(self.render_data.tex_shader.proj_mtx, 1, gl::FALSE, matrix.as_ptr() as _);

                if gl.gl.BindSampler.is_loaded() {
                    gl.gl.BindSampler(0, 0);
                }

                gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.render_data.vbo);
                gl.gl.EnableVertexAttribArray(self.render_data.tex_shader.position);
                gl.gl.EnableVertexAttribArray(self.render_data.tex_shader.uv);
                gl.gl.EnableVertexAttribArray(self.render_data.tex_shader.color);

                gl.gl.VertexAttribPointer(
                    self.render_data.tex_shader.position,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<DrawVert>() as _,
                    field_offset::<DrawVert, _, _>(|v| &v.pos) as _,
                );

                gl.gl.VertexAttribPointer(
                    self.render_data.tex_shader.uv,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<DrawVert>() as _,
                    field_offset::<DrawVert, _, _>(|v| &v.uv) as _,
                );

                gl.gl.VertexAttribPointer(
                    self.render_data.tex_shader.color,
                    4,
                    gl::UNSIGNED_BYTE,
                    gl::TRUE,
                    mem::size_of::<DrawVert>() as _,
                    field_offset::<DrawVert, _, _>(|v| &v.col) as _,
                );

                for draw_list in draw_data.draw_lists() {
                    let vtx_buffer = draw_list.vtx_buffer();
                    let idx_buffer = draw_list.idx_buffer();

                    gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.render_data.vbo);
                    gl.gl.BufferData(
                        gl::ARRAY_BUFFER,
                        (vtx_buffer.len() * mem::size_of::<DrawVert>()) as _,
                        vtx_buffer.as_ptr() as _,
                        gl::STREAM_DRAW,
                    );

                    gl.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.render_data.ebo);
                    gl.gl.BufferData(
                        gl::ELEMENT_ARRAY_BUFFER,
                        (idx_buffer.len() * mem::size_of::<DrawIdx>()) as _,
                        idx_buffer.as_ptr() as _,
                        gl::STREAM_DRAW,
                    );

                    for cmd in draw_list.commands() {
                        match cmd {
                            DrawCmd::Elements {
                                count,
                                cmd_params: DrawCmdParams { clip_rect: [x, y, z, w], texture_id, idx_offset, .. },
                            } => {
                                gl.gl.BindTexture(gl::TEXTURE_2D, texture_id.id() as _);

                                gl.gl.Scissor(
                                    (x * scale_w) as GLint,
                                    (fb_height - w * scale_h) as GLint,
                                    ((z - x) * scale_w) as GLint,
                                    ((w - y) * scale_h) as GLint,
                                );

                                let idx_size =
                                    if mem::size_of::<DrawIdx>() == 2 { gl::UNSIGNED_SHORT } else { gl::UNSIGNED_INT };

                                gl.gl.DrawElements(
                                    gl::TRIANGLES,
                                    count as _,
                                    idx_size,
                                    (idx_offset * mem::size_of::<DrawIdx>()) as _,
                                );
                            }
                            DrawCmd::ResetRenderState => {}
                            DrawCmd::RawCallback { .. } => {}
                        }
                    }
                }

                gl.gl.Disable(gl::SCISSOR_TEST);
            }
        }

        Ok(())
    }

    fn supports_vertex_draw(&self) -> bool {
        true
    }

    fn draw_triangle_list(
        &mut self,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult<()> {
        self.draw_arrays(gl::TRIANGLES, vertices, texture, shader)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl OpenGLRenderer {
    fn draw_arrays(
        &mut self,
        vert_type: GLenum,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult<()> {
        if vertices.is_empty() {
            return Ok(());
        }

        let texture_id = if let Some(texture) = texture {
            let gl_texture = texture
                .as_any()
                .downcast_ref::<OpenGLTexture>()
                .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))?;

            gl_texture.texture_id
        } else {
            0
        };

        unsafe { self.draw_arrays_tex_id(vert_type, vertices, texture_id, shader) }
    }

    unsafe fn draw_arrays_tex_id(
        &mut self,
        vert_type: GLenum,
        vertices: &[VertexData],
        mut texture: u32,
        shader: BackendShader,
    ) -> GameResult<()> {
        if let Some(gl) = &GL_PROC {
            match shader {
                BackendShader::Fill => {
                    self.render_data.fill_shader.bind_attrib_pointer(gl, self.render_data.vbo)?;
                }
                BackendShader::Texture => {
                    self.render_data.tex_shader.bind_attrib_pointer(gl, self.render_data.vbo)?;
                }
                BackendShader::WaterFill(scale, t, frame_pos) => {
                    self.render_data.fill_water_shader.bind_attrib_pointer(gl, self.render_data.vbo)?;
                    gl.gl.Uniform1f(self.render_data.fill_water_shader.scale, scale);
                    gl.gl.Uniform1f(self.render_data.fill_water_shader.time, t);
                    gl.gl.Uniform2f(self.render_data.fill_water_shader.frame_offset, frame_pos.0, frame_pos.1);
                    texture = self.render_data.surf_texture;
                }
            }

            gl.gl.BindTexture(gl::TEXTURE_2D, texture);
            gl.gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<VertexData>()) as _,
                vertices.as_ptr() as _,
                gl::STREAM_DRAW,
            );

            gl.gl.DrawArrays(vert_type, 0, vertices.len() as _);

            gl.gl.BindTexture(gl::TEXTURE_2D, 0);
            gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);

            Ok(())
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }
}

impl Drop for OpenGLRenderer {
    fn drop(&mut self) {
        *self.context_active.as_ref().borrow_mut() = false;
    }
}
