//! # Introduction
//! glium2 is a different take on the no-longer maintained glium.
//! This crate provides a safe API to use OpenGL.
//!
//! # Example
//!
//! Below is the standard `HelloTriangle` program.
//! ```
//! use glfw::{Action, Context, Key, WindowMode};
//! use glium2::{
//!     buffer::VertexBuffer,
//!     glfw::{self, OpenGlProfileHint, WindowEvent, WindowHint},
//!     glm,
//!     shader::{Program, Shader, ShaderType},
//!     uniforms, DrawMode, Renderer,
//! };
//!
//! let mut glfw = glfw::init_no_callbacks().expect("Failed to initialize GLFW");
//!
//! glfw.window_hint(WindowHint::ContextVersion(4, 6));
//! glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
//!
//! let (mut window, events) = glfw
//!     .create_window(800, 600, "Hello World!", WindowMode::Windowed)
//!     .expect("Failed to create window");
//!
//! window.make_current();
//! window.set_key_polling(true);
//!
//! Renderer::load_opengl_functions(|s| glfw.get_proc_address_raw(s));
//! let mut renderer = Renderer::new();
//! renderer.clear_color(glm::vec4(0.0, 0.0, 0.0, 1.0));
//!
//! let buffer = VertexBuffer::new(
//!     &vec![
//!         glm::vec2(0.0, 0.5),
//!         glm::vec2(0.5, 0.0),
//!         glm::vec2(-0.5, 0.0),
//!     ],
//!     None,
//! );
//!
//! let vertex_shader = Shader::new(
//!     r#"
//!         #version 460 core
//!         layout(location = 0) in vec2 vertexPosition;
//!
//!         void main() {
//!             gl_Position = vec4(vertexPosition, 0, 1);
//!         }
//!     "#,
//!     ShaderType::Vertex,
//! );
//!
//! let fragment_shader = Shader::new(
//!     r#"
//!         #version 460 core
//!
//!         out vec4 color;
//!
//!         void main() {
//!             color = vec4(1, 1, 1, 1);
//!         }
//!     "#,
//!     ShaderType::Fragment,
//! );
//!
//! let mut program = Program::new();
//! program
//!     .attach_and_link(vec![vertex_shader, fragment_shader])
//!     .expect("Failed to link program");
//!
//! # window.set_should_close(true);
//! while !window.should_close() {
//!     renderer.draw(&buffer, &program, DrawMode::Triangles, &uniforms! {});
//!
//!     window.swap_buffers();
//!     glfw.poll_events();
//!
//!     for (_, event) in glfw::flush_messages(&events) {
//!         if let WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
//!             window.set_should_close(true);
//!         }
//!     }
//! }
//!
//! ```

/// OpenGL buffer utilities
pub mod buffer;

/// Functions to generate matrices not supported by [`glm`]
pub mod matrix;

/// Graphical primitives
pub mod primitive;

/// The central structure of glium2
pub mod renderer;

/// OpenGL shader utilities
pub mod shader;

/// OpenGL types
pub mod types;

/// Shader uniforms
#[macro_use]
pub mod uniforms;

pub mod glm {
    pub use glm::*;
}

pub mod glfw {
    pub use glfw::*;
}

#[macro_use]
pub mod macros {
    pub use macros::*;
}

pub use renderer::{DrawMode, Renderer};
