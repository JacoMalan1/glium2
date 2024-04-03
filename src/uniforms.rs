pub trait Uniform: std::fmt::Debug {
    fn upload(&self, location: i32);
}

impl Uniform for glm::Matrix4<f32> {
    fn upload(&self, location: i32) {
        let data = self
            .as_array()
            .into_iter()
            .flat_map(|v| v.as_array())
            .map(|f| *f)
            .collect::<Vec<_>>();

        unsafe { gl::UniformMatrix4fv(location, 1, gl::FALSE, data.as_ptr().cast()) }
    }
}

impl<T> Uniform for &T
where
    T: Uniform,
{
    fn upload(&self, location: i32) {
        <T as Uniform>::upload(self, location);
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
