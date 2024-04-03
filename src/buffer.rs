use std::{marker::PhantomData, mem::MaybeUninit};

use gl::types::GLuint;

use crate::shader::{Vertex, VertexAttributeSpec};

#[derive(Debug)]
pub struct VertexBuffer<V> {
    vbo: u32,
    vao: u32,
    ibo: Option<u32>,
    vertex_count: usize,
    index_count: usize,
    _phantom: PhantomData<V>,
}

impl<V> Clone for VertexBuffer<V>
where
    V: Into<VertexData>,
{
    fn clone(&self) -> Self {
        let mut vertices: Vec<MaybeUninit<u8>> =
            vec![MaybeUninit::uninit(); self.vertex_count * std::mem::size_of::<V>()];
        let mut indices: Vec<MaybeUninit<u32>> = vec![MaybeUninit::uninit(); self.index_count];
        self.bind();

        unsafe {
            gl::GetBufferSubData(
                gl::ARRAY_BUFFER,
                0,
                vertices.len() as isize,
                vertices.as_mut_ptr().cast(),
            );
            if self.has_indices() {
                gl::GetBufferSubData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    0,
                    (indices.len() * std::mem::size_of::<u32>()) as isize,
                    indices.as_mut_ptr().cast(),
                );
            }
        };
        let vertices = vertices
            .into_iter()
            .map(|v| unsafe { v.assume_init() })
            .collect::<Vec<_>>();

        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, std::ptr::addr_of_mut!(vao));
            gl::BindVertexArray(vao);
        };

        let mut ibo = None;
        if self.has_indices() {
            let indices = indices
                .into_iter()
                .map(|i| unsafe { i.assume_init() })
                .collect::<Vec<_>>();
            let mut ibo_id = 0;
            ibo = Some(ibo_id);
            unsafe {
                gl::GenBuffers(1, std::ptr::addr_of_mut!(ibo_id));
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_id);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (self.index_count * std::mem::size_of::<u32>()) as isize,
                    indices.as_ptr().cast(),
                    gl::DYNAMIC_DRAW,
                );
            };
        }

        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, std::ptr::addr_of_mut!(vbo));
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertices.len() as isize,
                vertices.as_ptr().cast(),
                gl::DYNAMIC_DRAW,
            );
        };

        Self {
            vbo,
            vao,
            ibo,
            vertex_count: self.vertex_count,
            index_count: self.index_count,
            _phantom: PhantomData,
        }
    }
}

impl<V> VertexBuffer<V>
where
    V: Into<VertexData>,
{
    /// Creates a new vertex buffer from some vertices and, optionally, indices.
    pub fn new(vertices: &Vec<V>, indices: Option<&Vec<gl::types::GLuint>>) -> Self
    where
        V: Clone + std::fmt::Debug,
    {
        let mut id = 0;
        let mut vao = 0;
        let mut ibo = 0;
        let vertex_data = vertices
            .iter()
            .flat_map(|v| <V as Into<VertexData>>::into(v.clone()).data)
            .collect::<Vec<_>>();

        unsafe {
            gl::GenVertexArrays(1, std::ptr::addr_of_mut!(vao));
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, std::ptr::addr_of_mut!(id));
            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertex_data.len() as isize,
                vertex_data.as_ptr_range().start.cast(),
                gl::DYNAMIC_DRAW,
            );

            if let Some(ref indices) = indices {
                gl::GenBuffers(1, std::ptr::addr_of_mut!(ibo));
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (indices.len() * std::mem::size_of::<GLuint>()) as isize,
                    indices.as_ptr_range().start.cast(),
                    gl::DYNAMIC_DRAW,
                )
            }
        };

        Self {
            vbo: id,
            vertex_count: vertices.len(),
            vao,
            ibo: indices.as_ref().map(|_| ibo),
            index_count: indices.map_or_else(|| 0, |indices| indices.len()),
            _phantom: PhantomData,
        }
    }

    /// Binds all of the OpenGL buffers associated with the VertexBuffer
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            if let Some(ref ibo) = self.ibo {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, *ibo);
            }
        };
    }

    /// Returns the number of vertices in the VertexBuffer
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Returns whether the buffer contains an index buffer
    pub fn has_indices(&self) -> bool {
        self.ibo.is_some()
    }

    /// Returns the number of indices in the VertexBuffer.
    ///
    /// # Returns
    ///
    /// The number of indices, or `0` if there are none.
    pub fn index_count(&self) -> usize {
        self.index_count
    }

    /// Updates the contents of a vertex buffer.
    /// This function will call [`VertexBuffer::replace`] when appropriate.
    pub fn update_buffer(&mut self, vertices: &Vec<V>, indices: Option<&Vec<GLuint>>)
    where
        V: Clone,
    {
        self.bind();
        #[allow(clippy::unwrap_used)]
        if indices.is_some()
            && self.has_indices()
            && self.index_count() == indices.as_ref().unwrap().len()
            && self.vertex_count() == vertices.len()
        {
            unsafe { self.replace(vertices, indices) };
            return;
        }

        if indices.is_some() && !self.has_indices() {
            log::debug!("Allocating new index buffer for existing vertex buffer");
            let mut ibo = 0;
            unsafe {
                gl::GenBuffers(1, &mut ibo);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            };
            self.ibo = Some(ibo);
            self.index_count = 0;
        }

        if let Some(ref ibo) = self.ibo {
            if let Some(indices) = indices {
                unsafe {
                    gl::BufferData(
                        gl::ELEMENT_ARRAY_BUFFER,
                        (indices.len() * std::mem::size_of::<GLuint>()) as isize,
                        indices.as_ptr_range().start.cast(),
                        gl::DYNAMIC_DRAW,
                    );
                }

                self.index_count = indices.len();
            } else {
                log::debug!("Deleting unused index buffer after update with no indices");
                unsafe { gl::DeleteBuffers(1, ibo) };
                self.ibo = None;
                self.index_count = 0;
            }
        }

        let vertex_data = vertices
            .iter()
            .flat_map(|v| <V as Into<VertexData>>::into(v.clone()).data)
            .collect::<Vec<_>>();

        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertex_data.len() as isize,
                vertex_data.as_ptr_range().start.cast(),
                gl::DYNAMIC_DRAW,
            );
        };
        self.vertex_count = vertices.len();
    }

    /// Replaces the contents of the buffer(s) without reallocating the buffer.
    ///
    /// # Safety
    /// Callers of this function must ensure that the lengths and types of vertices and indices
    /// are exactly equal to [`VertexBuffer::vertex_count`] and [`VertexBuffer::index_count`]
    /// respectively.
    ///
    /// # Panics
    /// This function panics if the supplied indices are None, but the buffer previously
    /// contained indices and vice-versa.
    pub unsafe fn replace(&mut self, vertices: &Vec<V>, indices: Option<&Vec<GLuint>>)
    where
        V: Clone,
    {
        if indices.is_some() != self.has_indices() {
            panic!("Expected to replace indices, but none were given.")
        }

        self.bind();
        if let Some(indices) = indices {
            // We know from the previous if statement that we have an IBO
            #[allow(clippy::unwrap_used)]
            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                0,
                (indices.len() * std::mem::size_of::<GLuint>()) as isize,
                indices.as_ptr_range().start.cast(),
            )
        }
        let vertex_data = vertices
            .iter()
            .flat_map(|v| <V as Into<VertexData>>::into(v.clone()).data)
            .collect::<Vec<_>>();
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            vertex_data.len() as isize,
            vertex_data.as_ptr_range().start.cast(),
        );
    }
}

impl<V> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.vbo) };
        if let Some(ref ibo) = self.ibo {
            unsafe { gl::DeleteBuffers(1, ibo) };
        }
    }
}

/// A container for raw vertex data
pub struct VertexData {
    pub data: Vec<u8>,
}

impl From<glm::Vec2> for VertexData {
    fn from(value: glm::Vec2) -> Self {
        Self {
            data: value
                .as_array()
                .iter()
                .flat_map(|f| f.to_ne_bytes())
                .collect::<Vec<_>>(),
        }
    }
}

impl Vertex for glm::Vec2 {
    fn get_vertex_spec() -> crate::shader::VertexAttributeSpec {
        VertexAttributeSpec {
            layouts: vec![(
                2,
                gl::FLOAT,
                gl::FALSE,
                2 * std::mem::size_of::<f32>() as i32,
                0,
            )],
        }
    }
}
