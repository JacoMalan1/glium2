/// OpenGL buffer utilities
pub mod buffer;

/// Functions to generate matrices not supported by [`glm`]
pub mod matrix;

/// Graphical primitives
pub mod primitive;

pub mod renderer;

/// OpenGL shader utilities
pub mod shader;

pub mod types;

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
