use glfw::{Action, Context, Key, WindowMode};
use glium2::{
    buffer::VertexBuffer,
    glfw::{self, OpenGlProfileHint, WindowEvent, WindowHint},
    glm,
    shader::{Program, Shader, ShaderType},
    uniforms, DrawMode, Renderer,
};

fn main() {
    let mut glfw = glfw::init_no_callbacks().expect("Failed to initialize GLFW");

    glfw.window_hint(WindowHint::Samples(Some(8)));
    glfw.window_hint(WindowHint::ContextVersion(4, 6));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

    let (mut window, events) = glfw
        .create_window(800, 600, "Hello World!", WindowMode::Windowed)
        .expect("Failed to create window");

    window.make_current();
    window.set_key_polling(true);

    Renderer::load_opengl_functions(|s| glfw.get_proc_address_raw(s));
    let mut renderer = Renderer::new();
    renderer.clear_color(glm::vec4(0.0, 0.0, 0.0, 1.0));

    let buffer = VertexBuffer::new(
        &[
            glm::vec2(0.0, 0.5),
            glm::vec2(0.5, 0.0),
            glm::vec2(-0.5, 0.0),
        ],
        None,
    );

    let vertex_shader = Shader::new(
        r#"
            #version 460 core
            layout(location = 0) in vec2 vertexPosition;

            void main() {
                gl_Position = vec4(vertexPosition, 0, 1);
            }
        "#,
        ShaderType::Vertex,
    );

    let fragment_shader = Shader::new(
        r#"
            #version 460 core

            out vec4 color;

            void main() {
                color = vec4(1, 1, 1, 1);
            }
        "#,
        ShaderType::Fragment,
    );

    let mut program = Program::new();
    program
        .attach_and_link(vec![vertex_shader, fragment_shader])
        .expect("Failed to link program");

    while !window.should_close() {
        renderer.draw(&buffer, &program, DrawMode::Triangles, &uniforms! {});

        window.swap_buffers();
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                window.set_should_close(true);
            }
        }
    }
}
