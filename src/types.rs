pub trait OpenGLType {
    fn opengl_type() -> u32;
}

impl OpenGLType for f32 {
    fn opengl_type() -> u32 {
        gl::FLOAT
    }
}

impl OpenGLType for u32 {
    fn opengl_type() -> u32 {
        gl::UNSIGNED_INT
    }
}

impl OpenGLType for u8 {
    fn opengl_type() -> u32 {
        gl::UNSIGNED_BYTE
    }
}

impl OpenGLType for i32 {
    fn opengl_type() -> u32 {
        gl::INT
    }
}
