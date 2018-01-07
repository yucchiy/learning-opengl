extern crate glfw;
extern crate gl;

use self::gl::types::*;
use self::glfw::{Context, Key, Action};

use std::sync::mpsc::Receiver;
use std::ffi::CString;
use std::ptr;
use std::str;
use std::mem;
use std::os::raw::c_void;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 410 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 410 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 0.0f, 0.0f, 1.0f);
    }
"#;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 1));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw.create_window(WIDTH, HEIGHT, "This is a window", glfw::WindowMode::Windowed)
        .expect("Failed to create a GLFW window.");
    
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader_program, vao) = unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vertex = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vertex.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(vertex_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_fragment = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_fragment.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(fragment_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        // check for linking errors
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(shader_program, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        let vertices: [f32; 18] = [
            -0.9, -0.5, 0.0,
            -0.0, -0.5, 0.0,
            -0.45, 0.5, 0.0,
            0.0, -0.5, 0.0,
            0.9, -0.5, 0.0, 
            0.45, 0.5, 0.0
        ];

        let (mut vbo, mut vao) = (0, 0);

        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        (shader_program, vao)
    };

    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe { gl::Viewport(0, 0, width, height) }
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            _ => {}
        }
    }
}

