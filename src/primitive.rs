use crate::{
    buffer::VertexBuffer,
    renderer::{DrawMode, Renderer},
    shader::{self, Program, Vertex, VertexAttributeSpec},
    uniforms::Uniforms,
};
use glm::{Vec3, Vec4};

/// A trait representing any primitive graphical object
pub trait Primitive {
    type Vertex: Vertex;

    /// Turns [`self`] into a mesh by creating a vertex buffer and specifying a [`DrawMode`]
    fn into_mesh(self) -> Mesh<Self::Vertex>;
    fn buffer(&self) -> &VertexBuffer<Self::Vertex>;
    fn buffer_mut(&mut self) -> &mut VertexBuffer<Self::Vertex>;
    fn draw_mode(&self) -> DrawMode;
}

macro_rules! colour_vertex {
    ( $x: expr, $y: expr, $z: expr; $r: expr, $g: expr, $b: expr, $a: expr ) => {{
        crate::primitive::ColorVertex {
            position: glm::Vec3::new($x, $y, $z),
            color: glm::Vec4::new($r, $g, $b, $a),
        }
    }};
}

/// A circle made of triangles
#[derive(Debug, Clone)]
pub struct Circle {
    center: Vec3,
    radius: f32,
    segments: i32,
    vertex_buffer: VertexBuffer<ColorVertex>,
}

impl Circle {
    fn calculate_vertices(center: Vec3, radius: f32, segments: i32) -> Vec<ColorVertex> {
        let mut vertices = vec![center];
        let delta_theta = 2.0 * std::f32::consts::PI / segments as f32;
        for i in 0..segments {
            let angle = -i as f32 * delta_theta;
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            vertices.push(glm::vec3(x, y, center.z) + center);
        }
        vertices.push(glm::vec3(radius + center.x, center.y, center.z));

        vertices
            .into_iter()
            .map(|v| ColorVertex {
                position: v,
                color: glm::vec4(1.0, 1.0, 1.0, 1.0),
            })
            .collect::<Vec<_>>()
    }

    /// Constructs a new circle from a center, radius and number of segments
    pub fn new(center: Vec3, radius: f32, segments: i32) -> Self {
        let vertices = Self::calculate_vertices(center, radius, segments);
        let buffer = VertexBuffer::new(&vertices, None);

        Self {
            center,
            radius,
            segments,
            vertex_buffer: buffer,
        }
    }

    pub fn center(&self) -> &Vec3 {
        &self.center
    }

    pub fn set_center(&mut self, center: Vec3) -> Vec3 {
        let old = std::mem::replace(&mut self.center, center);
        let vertices = Self::calculate_vertices(self.center, self.radius, self.segments);
        self.buffer_mut().update_buffer(&vertices, None);
        old
    }
}

impl Primitive for Circle {
    type Vertex = ColorVertex;

    fn into_mesh(self) -> Mesh<Self::Vertex> {
        let vertices = Self::calculate_vertices(self.center, self.radius, self.segments);

        Mesh {
            buffer: VertexBuffer::new(&vertices, None),
            draw_mode: self.draw_mode(),
        }
    }

    fn buffer(&self) -> &VertexBuffer<Self::Vertex> {
        &self.vertex_buffer
    }

    fn buffer_mut(&mut self) -> &mut VertexBuffer<Self::Vertex> {
        &mut self.vertex_buffer
    }

    fn draw_mode(&self) -> DrawMode {
        DrawMode::TriangleFan
    }
}

/// A 2D square represented by a position and a side length
pub struct Square {
    position: Vec3,
    side_length: f32,
    vertex_buffer: VertexBuffer<ColorVertex>,
}

impl Square {
    /// Constructs a new Square
    ///
    /// # Params
    /// `position` - The top left corner of the square.
    ///
    /// `side_length` - The length of each side.
    pub fn new(position: Vec3, side_length: f32) -> Self {
        let (vertices, indices) = Self::calculate_vertices(position, side_length);
        Self {
            position,
            side_length,
            vertex_buffer: VertexBuffer::new(&vertices, Some(&indices)),
        }
    }

    /// Returns the square's position
    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    /// Sets the position and returns the old position
    pub fn set_position(&mut self, position: Vec3) -> glm::Vec3 {
        let old = std::mem::replace(&mut self.position, position);
        let (vertices, indices) = Self::calculate_vertices(self.position, self.side_length);
        self.buffer_mut().update_buffer(&vertices, Some(&indices));
        old
    }

    /// Returns the square's side length
    pub fn side_length(&self) -> f32 {
        self.side_length
    }

    /// Sets the side length and returns the old side length
    pub fn set_side_length(&mut self, side_length: f32) -> f32 {
        let old = std::mem::replace(&mut self.side_length, side_length);
        let (vertices, indices) = Self::calculate_vertices(self.position, self.side_length);
        self.buffer_mut().update_buffer(&vertices, Some(&indices));
        old
    }

    fn calculate_vertices(position: Vec3, side_length: f32) -> (Vec<ColorVertex>, Vec<u32>) {
        (
            vec![
                colour_vertex!(
                    position.x,
                    position.y,
                    position.z;
                    1.0,
                    1.0,
                    1.0,
                    1.0
                ),
                colour_vertex!(
                    position.x,
                    position.y + side_length,
                    position.z;
                    1.0,
                    1.0,
                    1.0,
                    1.0
                ),
                colour_vertex!(
                    position.x + side_length,
                    position.y + side_length,
                    position.z;
                    1.0,
                    1.0,
                    1.0,
                    1.0
                ),
                colour_vertex!(
                    position.x + side_length,
                    position.y,
                    position.z;
                    1.0,
                    1.0,
                    1.0,
                    1.0
                ),
            ],
            vec![0, 1, 2, 0, 2, 3],
        )
    }
}

impl Primitive for Square {
    type Vertex = ColorVertex;

    fn into_mesh(self) -> Mesh<Self::Vertex> {
        let (vertices, indices) = Self::calculate_vertices(self.position, self.side_length);
        Mesh {
            buffer: VertexBuffer::new(&vertices, Some(&indices)),
            draw_mode: DrawMode::Triangles,
        }
    }

    fn draw_mode(&self) -> DrawMode {
        DrawMode::Triangles
    }

    fn buffer(&self) -> &VertexBuffer<Self::Vertex> {
        &self.vertex_buffer
    }

    fn buffer_mut(&mut self) -> &mut VertexBuffer<Self::Vertex> {
        &mut self.vertex_buffer
    }
}

pub struct Mesh<V> {
    buffer: VertexBuffer<V>,
    draw_mode: DrawMode,
}

impl<V> Mesh<V>
where
    V: Vertex,
{
    pub fn buffer(&self) -> &VertexBuffer<V> {
        &self.buffer
    }

    pub fn draw(&self, renderer: &mut Renderer, shader_program: &Program, uniforms: &Uniforms) {
        renderer.draw(self.buffer(), shader_program, self.draw_mode, uniforms)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ColorVertex {
    pub position: Vec3,
    pub color: Vec4,
}

impl From<ColorVertex> for crate::buffer::VertexData {
    fn from(vertex: ColorVertex) -> crate::buffer::VertexData {
        let mut data = Vec::new();
        data.extend_from_slice(vertex.position.as_array());
        data.extend_from_slice(vertex.color.as_array());
        crate::buffer::VertexData {
            data: data
                .into_iter()
                .flat_map(|f| f.to_ne_bytes())
                .collect::<Vec<_>>(),
        }
    }
}

impl Vertex for ColorVertex {
    fn get_vertex_spec() -> shader::VertexAttributeSpec {
        VertexAttributeSpec {
            layouts: vec![
                (
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    7 * std::mem::size_of::<f32>() as i32,
                    0,
                ),
                (
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    7 * std::mem::size_of::<f32>() as i32,
                    3 * std::mem::size_of::<f32>(),
                ),
            ],
        }
    }
}
