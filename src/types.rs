/// A trait representing Rust types that correspond with OpenGL types
pub trait OpenGLType {
    fn opengl_type() -> u32;
}

impl OpenGLType for f32 {
    fn opengl_type() -> u32 {
        gl::FLOAT
    }
}

impl OpenGLType for f64 {
    fn opengl_type() -> u32 {
        gl::DOUBLE
    }
}

impl OpenGLType for u32 {
    fn opengl_type() -> u32 {
        gl::UNSIGNED_INT
    }
}

impl OpenGLType for i32 {
    fn opengl_type() -> u32 {
        gl::INT
    }
}

impl OpenGLType for u8 {
    fn opengl_type() -> u32 {
        gl::UNSIGNED_BYTE
    }
}

impl OpenGLType for u16 {
    fn opengl_type() -> u32 {
        gl::UNSIGNED_SHORT
    }
}

impl OpenGLType for i16 {
    fn opengl_type() -> u32 {
        gl::SHORT
    }
}
