/// A trait for types that can be used as OpenGL uniform values
pub trait Uniform: std::fmt::Debug {
    fn upload(&self, location: i32);
}

impl Uniform for glm::Matrix4<f32> {
    fn upload(&self, location: i32) {
        let data = self
            .as_array()
            .iter()
            .flat_map(|v| v.as_array())
            .copied()
            .collect::<Vec<_>>();

        unsafe { gl::UniformMatrix4fv(location, 1, gl::FALSE, data.as_ptr().cast()) }
    }
}

impl Uniform for glm::Vector2<f32> {
    fn upload(&self, location: i32) {
        let data = self.as_array();
        unsafe { gl::Uniform2fv(location, 1, data.as_ptr().cast()) }
    }
}

impl Uniform for glm::Vector3<f32> {
    fn upload(&self, location: i32) {
        let data = self.as_array();
        unsafe { gl::Uniform3fv(location, 1, data.as_ptr().cast()) }
    }
}

impl Uniform for glm::Vector4<f32> {
    fn upload(&self, location: i32) {
        let data = self.as_array();
        unsafe { gl::Uniform4fv(location, 1, data.as_ptr().cast()) }
    }
}

impl Uniform for i32 {
    fn upload(&self, location: i32) {
        unsafe { gl::Uniform1i(location, *self) }
    }
}

impl Uniform for f64 {
    fn upload(&self, location: i32) {
        unsafe { gl::Uniform1d(location, *self) }
    }
}

impl Uniform for f32 {
    fn upload(&self, location: i32) {
        unsafe { gl::Uniform1f(location, *self) }
    }
}

#[derive(Debug)]
pub struct Uniforms {
    pub data: Vec<(i32, Box<dyn Uniform>)>,
}

impl Uniforms {
    pub fn upload_all(&self) {
        self.data.iter().for_each(|u| u.1.upload(u.0))
    }
}

#[macro_export]
macro_rules! uniforms {
    () => {{
        $crate::uniforms::Uniforms {
            data: vec![]
        }
    }};

    ( $program: ident => { $($name:literal : $uniform:expr),* } ) => {{
        let mut data = Vec::new();
        $(
            let location = $program.get_uniform_location($name);
            let b: std::boxed::Box<dyn $crate::uniforms::Uniform> = std::boxed::Box::new($uniform);
            data.push((location, b));
        )*

        $crate::uniforms::Uniforms {
            data
        }
    }};
}
