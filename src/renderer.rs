use crate::{
    buffer::VertexBuffer,
    shader::{Program, Vertex},
    uniforms::Uniforms,
};
use glm::Vec4;
use std::{
    os::raw::c_void,
    ptr::{null, slice_from_raw_parts},
};

#[derive(Debug, Copy, Clone)]
pub enum DrawMode {
    Triangles,
    TriangleStrip,
    TriangleFan,
}

impl From<DrawMode> for u32 {
    fn from(mode: DrawMode) -> u32 {
        match mode {
            DrawMode::Triangles => gl::TRIANGLES,
            DrawMode::TriangleStrip => gl::TRIANGLE_STRIP,
            DrawMode::TriangleFan => gl::TRIANGLE_FAN,
        }
    }
}

pub enum CullingMode {
    Clockwise,
    CounterClockwise,
    None,
}

pub struct Renderer {
    clear_color: Vec4,
    clear_depth: f64,
}

impl Renderer {
    /// Constructs a new Renderer.
    ///
    /// This function must be called AFTER [`Renderer::load_opengl_functions`]
    pub fn new() -> Self {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(Self::debug_callback), null());
        };

        Self {
            clear_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
            clear_depth: 0.0,
        }
    }

    /// Sets the clear color for the renderer.
    pub fn clear_color(&mut self, color: Vec4) {
        self.clear_color = color;
    }

    /// Sets the clear depth for the renderer
    pub fn clear_depth(&mut self, depth: f64) {
        // if unsafe { gl::IsEnabled(gl::DEPTH_TEST) != gl::TRUE } {
        //     unsafe {
        //         gl::Enable(gl::DEPTH_TEST);
        //         gl::DepthFunc(gl::GREATER);
        //     };
        // }
        self.clear_depth = depth;
    }

    /// Loads the function table for OpenGL.
    ///
    /// Must be called before constructing a renderer or any other object in this library
    pub fn load_opengl_functions<F>(load_with: F)
    where
        F: FnMut(&'static str) -> *const c_void,
    {
        gl::load_with(load_with);
    }

    pub fn cull_faces(&mut self, culling_mode: CullingMode) {
        match culling_mode {
            CullingMode::Clockwise => unsafe {
                gl::Enable(gl::CULL_FACE);
                gl::CullFace(gl::BACK);
                gl::FrontFace(gl::CCW);
            },
            CullingMode::CounterClockwise => unsafe {
                gl::Enable(gl::CULL_FACE);
                gl::CullFace(gl::BACK);
                gl::FrontFace(gl::CW);
            },
            CullingMode::None => {
                unsafe { gl::Disable(gl::CULL_FACE) };
            }
        }
    }

    extern "system" fn debug_callback(
        _source: u32,
        ty: u32,
        _id: u32,
        severity: u32,
        length: i32,
        message: *const i8,
        _user_param: *mut c_void,
    ) {
        let message = unsafe {
            slice_from_raw_parts(message, length as usize)
                .as_ref()
                .expect("Failed to create slice from log message")
        }
        .iter()
        .map(|c| *c as u8)
        .collect::<Vec<_>>();

        let message = String::from_utf8(message).expect("Debug message was invalid String");

        if ty == gl::DEBUG_TYPE_ERROR {
            log::error!(
                "Debug Callback: ** GL ERROR ** type = {ty}, severity = {severity}, message = {message}\n",
            );
        } else {
            log::debug!(
                "Debug Callback: type = {ty}, severity = {severity}, message = {message}\n",
            );
        }
    }

    /// Clears the pixel buffer currently being drawn to
    pub fn clear(&self) {
        unsafe {
            gl::ClearColor(
                self.clear_color.x,
                self.clear_color.y,
                self.clear_color.z,
                self.clear_color.w,
            );
            gl::ClearDepth(self.clear_depth);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        };
    }

    /// Draws a buffer to the screen
    pub fn draw<V: Vertex>(
        &self,
        buffer: &VertexBuffer<V>,
        shader_program: &Program,
        mode: DrawMode,
        uniforms: &Uniforms,
    ) {
        buffer.bind();
        shader_program.bind();
        uniforms.upload_all();

        let vertex_spec = <V as Vertex>::get_vertex_spec();
        for i in 0..vertex_spec.layouts.len() {
            unsafe {
                gl::EnableVertexAttribArray(i as u32);
            }
        }

        vertex_spec
            .layouts
            .iter()
            .enumerate()
            .for_each(|(index, layout)| unsafe {
                let (size, ty, normalized, stride, offset) = *layout;
                gl::VertexAttribPointer(
                    index as u32,
                    size,
                    ty,
                    normalized,
                    stride,
                    offset as *const c_void,
                )
            });

        if buffer.has_indices() {
            unsafe {
                gl::DrawElements(
                    mode.into(),
                    buffer.index_count() as i32,
                    gl::UNSIGNED_INT,
                    null(),
                )
            }
        } else {
            unsafe { gl::DrawArrays(mode.into(), 0, buffer.vertex_count() as i32) };
        }
        for i in 0..vertex_spec.layouts.len() {
            unsafe {
                gl::DisableVertexAttribArray(i as u32);
            }
        }
    }
}
