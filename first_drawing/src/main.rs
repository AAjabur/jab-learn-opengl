extern crate sdl2;
extern crate gl;


use gl::types::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

fn compile_shaders(shader_type: u32, source_code: &str) -> GLuint {
    unsafe {
        let id = gl::CreateShader(shader_type);
        let c_str = std::ffi::CString::new(source_code).unwrap();

        gl::ShaderSource(
            id, 
            1, 
            &c_str.as_ptr(),
            ptr::null()
        );

        gl::CompileShader(id);

        let mut result: i32 = 0;
        gl::GetShaderiv(
            id, 
            gl::COMPILE_STATUS, 
            &mut result
        );
        if result as u8 == gl::FALSE {
            let mut length: i32 = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut length);

            let mut error_msg_buffer: Vec<u8> = Vec::with_capacity(length as usize);

            gl::GetShaderInfoLog(
                id, 
                length, 
                &mut length, 
                error_msg_buffer.as_mut_ptr() as *mut i8
            );

            let error_msg = String::from_utf8_lossy(
                &error_msg_buffer as &Vec<u8>
            );

            let shader_type_str = if shader_type == gl::VERTEX_SHADER {"Vertex shader "} else {"Fragment shader"};
                println!("{} compilation failed {}", shader_type_str, error_msg);

        }

        return id;
    }
}

fn create_shaders(vertex_shader: &str, fragment_shader: &str) -> u32 {
    unsafe {
        let program = gl::CreateProgram();

        // Compila os shaders de vertice, um código que irá rodar para cada um dos vértices que
        // foram definidos anteriormente
        let vs = compile_shaders(gl::VERTEX_SHADER, vertex_shader);
        // Compile os shaders de fragments (pixels), esse código irá rodar para cada um dos pixels
        // que estão dentro dos vértices
        let fs = compile_shaders(gl::FRAGMENT_SHADER, fragment_shader);

        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        gl::ValidateProgram(program);

        gl::DeleteShader(vs);
        gl::DeleteShader(fs);

        return program;
    }

}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(4, 4);

    let window = video_subsystem.window("Window", 800, 600)
        .opengl()
        .build()
        .unwrap();

    // Unlike the other example above, nobody created a context for your window, so you need to create one.
    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    
    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (4, 4));

    let (fb_width, fb_height) = window.drawable_size();
    unsafe {
        gl::Viewport(0, 0, fb_width as i32, fb_height as i32);
    }

    let mut vbo: GLuint = 0; // Vertex Buffer Object
    let mut vao: GLuint = 0; // Vertex Array Object
    let mut ebo: GLuint = 0; // Element Buffer Object

    // Perceba que apesar de dividirmos em 3 linhas
    // Esse é apenas um array, ou seja, futuramente precisamos 
    // de alguma forma alertar o openGL que são 3 posições de vértices
    // em 2D 
    let positions: [f32; 8] = [
        -0.5, -0.5, // Bottom left
         0.5, -0.5, // Bottom right
        -0.5,  0.5, // Top left
         0.5,  0.5, // Top right
    ];
    
    // Define o Element Buffer Object, os indices que compõem cada objeto, eles são importantes
    // para diminuir a repetição de posições iguais na definição de multiplos objetos
    let indices: [u32; 6] = [
        1, 0, 2, // Triângulo da esquerda
        1, 3, 2  // Triângulo da direita
    ];

    let mut event_pump = sdl_context.event_pump().unwrap();
    let shader: u32;
    unsafe {
        // Cria o Vertex Array Object (VAO), buffer que irá armazenar todas as outras definições
        // que criarmos posteriormente
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);         // Crie um vertex buffer object (VBO) de tamanho 1, 
        // armazene o ID desse buffer na variável buffer
        gl::GenBuffers(1, &mut ebo);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo); // Seleciona o buffer pelo ID dele, sabendo que é
        // um buffer que representa um array
        gl::BufferData(
            gl::ARRAY_BUFFER,                   // Esse buffer é simplesmente um arractxy
            (positions.len() * size_of::<f32>()) as GLsizeiptr, // Tamanho do buffer
            positions.as_ptr() as *const _,      // ponteiro das posições dos vertices
            gl::STATIC_DRAW                     // Apenas uma dica de que será alterado pouco
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo); // Selectiona a variável ebo como o buffer
        // de tipo ELEMENT_ARRAY_BUFFER selecionado, apenas um buffer de um tipo pode estar
        // selecionado ao mesmo tempo, mas podemos selecionar múltiplos buffers de diferentes tipos
        // ao mesmo tempo
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * size_of::<u32>()) as GLsizeiptr,
            indices.as_ptr() as * const _,
            gl::STATIC_DRAW
        );

        gl::VertexAttribPointer(
            0,          // Index: Estamos definindo o atributo genérico 0 de nossos vértices
            2,          // Size: Esse atributo tem tamanho 2, porque é uma posição em 2D que tem dois
                        // floats
            gl::FLOAT,  // Type: cada parte do atributo é um float
            false as u8,      // Normalized: não queremos que a GPU normalize pois já está normalizado 
            2 * size_of::<f32>() as GLsizei,          // Stride: defasagem em Bytes entre duas definições desse atributo
            0 as *const _  // Pointer: posição em Bytes da primeira aparição desse atributo
        ); // Ao chamar esse função o OpenGL linka o VBO selecionado à esse atributo e armazena
        // isso no VAO, então já podemos fazer unbind desse VBO

        gl::BindBuffer(gl::ARRAY_BUFFER, 0); // Unbind do VBO pois já armazenamos no VAO na chamada
        // acima

        gl::EnableVertexAttribArray(0); // Habilita o atributo genérico 0
        
        gl::BindVertexArray(0); // Unbind do VAO
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0); // Unbind do EBO DEPOIS do unbind do VAO, se
        // fizessemos antes ele faria unbind dentro daquele VAO e quando fizessemos vind denovo no
        // VAO ele não selecionaria esse EBO

        let vertex_shader = "#version 330 core
            layout(location = 0) in vec4 position;

            void main() {
                gl_Position = position;
            }";
        
        let fragment_shader = "#version 330 core
            layout(location = 0) out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }";
        shader = create_shaders(vertex_shader, fragment_shader);
    }
    'running: loop {
        unsafe {
            gl::UseProgram(shader);
            gl::ClearColor(0.0, 1.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(vao);
            gl::DrawElements(
                gl::TRIANGLES, 
                6, 
                gl::UNSIGNED_INT, 
                0 as *const _
            );
            gl::BindVertexArray(0);
            // gl::DrawArrays(gl::TRIANGLES, 0, 3); // Desenha o buffer que foi binded anteriormente
        }

        window.gl_swap_window();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
use std::{f32, ptr};
use std::mem::size_of;

