/// Constructs an orthographic projection matrix.
pub fn ortho(
    left: f32,
    right: f32,
    near: f32,
    far: f32,
    top: f32,
    bottom: f32,
) -> glm::Matrix4<f32> {
    glm::Matrix4::new(
        glm::vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
        glm::vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
        glm::vec4(0.0, 0.0, -2.0 / (far - near), 0.0),
        glm::vec4(
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            -(far + near) / (far - near),
            1.0,
        ),
    )
}
