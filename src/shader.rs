use std::{
    ffi::{c_char, CString},
    io::Read,
};

/// An abstraction for the concept of a Vertex Attribute Array
/// Usage of this struct outside of the library is currently unsafe, since
/// the memory safety of the GPU buffer associated depends on the user supplying correct values.
pub struct VertexAttributeSpec {
    pub(crate) layouts: Vec<(i32, u32, u8, i32, usize)>,
}

impl VertexAttributeSpec {
    /// Constructs an empty vertex attribute specification
    pub fn new() -> Self {
        Self { layouts: vec![] }
    }

    /// Adds a layout to the vertex specification
    ///
    /// # Safety
    /// In this function, the safety is really memory safety on the GPU.
    /// Users of this function must ensure that vertex data is really organized the way
    /// the parameters to this function claims.
    ///
    /// # Params
    ///
    /// `count` - The number of components in the layout
    /// `ty` - The type of data in this layout
    /// `normalized` - Whether the data is normalized
    /// `stride` - The number of bytes between elements in the layout
    /// `offset` - The number of bytes from the start of the buffer to the first element in this
    /// layout
    /// ``
    pub unsafe fn push_layout(
        &mut self,
        count: i32,
        ty: u32,
        normalized: bool,
        stride: i32,
        offset: usize,
    ) {
        self.layouts.push((
            count,
            ty,
            if normalized { gl::TRUE } else { gl::FALSE },
            stride,
            offset,
        ));
    }
}

/// A trait representing a single vertex usable in an OpenGL buffer
pub trait Vertex: Into<crate::buffer::VertexData> + Clone {
    /// Calculates the `glVertexAttribPointer` specification for a vertex of this type
    fn get_vertex_spec() -> VertexAttributeSpec;
}

/// The linking state of a GLSL program
pub enum ProgramState {
    /// The program has not been linked
    Unlinked,
    /// The program has been linked
    Linked,
    /// An error occurred during the linking process
    LinkerError(String),
}

/// A GLSL shader program
pub struct Program {
    id: u32,
    linked: ProgramState,
}

impl Program {
    /// Generates a blank shader program
    pub fn new() -> Self {
        Self {
            id: unsafe { gl::CreateProgram() },
            linked: ProgramState::Unlinked,
        }
    }

    /// Attaches shaders and links program
    pub fn attach_and_link<S>(
        &mut self,
        shaders: Vec<Shader<S>>,
    ) -> Result<(), ShaderCompilationError>
    where
        S: AsRef<str>,
    {
        for ref mut shader in shaders {
            shader.compile()?;
            unsafe { gl::AttachShader(self.id, shader.id) };
        }

        unsafe { gl::LinkProgram(self.id) };
        let mut link_status = 0;
        unsafe {
            gl::GetProgramiv(
                self.id,
                gl::LINK_STATUS,
                std::ptr::addr_of_mut!(link_status),
            );
        };

        if link_status != gl::TRUE as i32 {
            let mut info_log_length = 0;
            unsafe {
                gl::GetProgramiv(
                    self.id,
                    gl::INFO_LOG_LENGTH,
                    std::ptr::addr_of_mut!(info_log_length),
                )
            };
            let mut buffer = Vec::with_capacity(info_log_length as usize);
            let mut bytes_written = 0;
            unsafe {
                gl::GetProgramInfoLog(
                    self.id,
                    info_log_length,
                    std::ptr::addr_of_mut!(bytes_written),
                    buffer.as_mut_ptr_range().start,
                );
                buffer.set_len(bytes_written as usize);
            };

            let error = ShaderCompilationError(
                String::from_utf8(buffer.into_iter().map(|c| c as u8).collect::<Vec<_>>())
                    .expect("Info log is not a valid String"),
            );

            self.linked = ProgramState::LinkerError(error.0.clone());
            return Err(error);
        }

        self.linked = ProgramState::Linked;
        Ok(())
    }

    /// Sets `self` as the currently active program to be used for drawing.
    pub fn bind(&self) {
        unsafe { gl::UseProgram(self.id) };
    }

    pub fn get_uniform_location(&self, name: &str) -> i32 {
        let name_bytes = name
            .as_bytes()
            .bytes()
            .map(|b| b.unwrap() as i8)
            .chain(vec![0 as i8])
            .collect::<Vec<_>>();
        unsafe { gl::GetUniformLocation(self.id, name_bytes.as_ptr_range().start.cast()) }
    }

    pub fn state(&self) -> &ProgramState {
        &self.linked
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        // SAFETY: We are being dropped, so we can destroy the program we correspond with
        unsafe { gl::DeleteProgram(self.id) };
    }
}

/// The compilation state of an individual shader
#[derive(Debug, Clone)]
pub enum ShaderState<S>
where
    S: AsRef<str>,
{
    /// The shader is uncompiled with source code `S`
    Uncompiled(S),

    /// The shader failed to be compiled with some [`ShaderCompilationError`]
    CompilationError(ShaderCompilationError),

    /// The shader was successfully compiled
    Compiled,
}

/// A shader compilation error represented as a [`String`]
#[derive(Debug, Clone)]
pub struct ShaderCompilationError(String);

pub struct Shader<S>
where
    S: AsRef<str>,
{
    id: u32,
    state: ShaderState<S>,
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

impl<S> Shader<S>
where
    S: AsRef<str>,
{
    pub fn new(source: S, shader_type: ShaderType) -> Self {
        let id = unsafe {
            gl::CreateShader(match shader_type {
                ShaderType::Fragment => gl::FRAGMENT_SHADER,
                ShaderType::Vertex => gl::VERTEX_SHADER,
            })
        };

        Self {
            id,
            state: ShaderState::Uncompiled(source),
        }
    }

    fn compile(&mut self) -> Result<(), ShaderCompilationError> {
        match self.state {
            ShaderState::Compiled => Ok(()),
            ShaderState::CompilationError(ref err) => Err(err.clone()),
            ShaderState::Uncompiled(ref source) => {
                let len_ptr = source.as_ref().len() as i32;
                let source_cstring = CString::new(source.as_ref().as_bytes())
                    .expect("Source code is not a valid CString");
                let source_ptr: *const *const c_char = &source_cstring.as_ptr();

                // SAFETY: source_ptr and len_ptr are both valid pointers at this time.
                // OpenGL should not write to these, so it doesn't matter that the underlying
                // references are shared.
                unsafe { gl::ShaderSource(self.id, 1, source_ptr, std::ptr::addr_of!(len_ptr)) };
                let mut compile_status = 0;

                // SAFETY: compile_status has not been dropped yet, and this function call won't
                // outlive it.
                unsafe {
                    gl::GetShaderiv(
                        self.id,
                        gl::COMPILE_STATUS,
                        std::ptr::addr_of_mut!(compile_status),
                    )
                }
                if compile_status != gl::FALSE as i32 {
                    let mut info_log_length = 0;
                    unsafe {
                        gl::GetShaderiv(
                            self.id,
                            gl::INFO_LOG_LENGTH,
                            std::ptr::addr_of_mut!(info_log_length),
                        )
                    };

                    let mut bytes_written = 0;
                    let mut log_buffer = vec![0i8; info_log_length as usize];
                    unsafe {
                        gl::GetShaderInfoLog(
                            self.id,
                            info_log_length,
                            std::ptr::addr_of_mut!(bytes_written),
                            log_buffer.as_mut_ptr_range().start,
                        );
                    };

                    let error = ShaderCompilationError(
                        String::from_utf8(
                            log_buffer.into_iter().map(|c| c as u8).collect::<Vec<_>>(),
                        )
                        .expect("Shader info log is not a valid String"),
                    );

                    self.state = ShaderState::CompilationError(error.clone());
                    return Err(error);
                }

                self.state = ShaderState::Compiled;
                Ok(())
            }
        }
    }
}

impl<S> Drop for Shader<S>
where
    S: AsRef<str>,
{
    fn drop(&mut self) {
        // SAFETY: We are being dropped, so we can destroy the shader we correspond with
        unsafe { gl::DeleteShader(self.id) };
    }
}
