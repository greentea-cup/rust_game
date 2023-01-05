mod config;
mod ext;
mod logger;

use std::{
    collections::HashMap,
    fs::{create_dir_all, read_to_string},
    mem::size_of,
    path::Path,
};

use ext::gl::{gl_init, ContextExt};
use glow::{Context, HasContext, NativeShader, Program};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::{Mod, Scancode},
};

fn main() {
    unsafe { run() }
}

unsafe fn run() {
    let width = 854;
    let height = 480;
    let env_dir = Path::new(".")
        .canonicalize()
        .expect("Cannot access environment directory");
    let log_dir = env_dir.join("log");
    let data_dir = env_dir.join("data");
    let shaders_dir = data_dir.join("shaders");
    let vertex_shader = shaders_dir.join("vertex.glsl");
    let fragment_shader = shaders_dir.join("fragment.glsl");

    create_dir_all(log_dir.as_path()).expect("Cannot create logs directory");
    logger::setup_logger(log_dir.as_path()).expect("Cannot setup logger");
    log::set_max_level(log::LevelFilter::Info);

    let (gl, window, mut event_loop, _context) = gl_init("Title", width, height);

    let vertex_shader_source =
        read_to_string(vertex_shader.as_path()).expect("Cannot get vertex shader");
    let fragment_shader_source =
        read_to_string(fragment_shader.as_path()).expect("Cannot get fragment shader");
    let vertex_sources = vec![vertex_shader_source]; // here is all vertex shaders
    let fragment_sources = vec![fragment_shader_source]; // here is all fragment shaders

    let shaders = compile_shaders(&gl, vertex_sources, fragment_sources);
    let program = link_shaders(&gl, shaders);

    // let verticies = [
    //     // x, y, z
    //     0.5f32, -0.5, -1.0, // Bottom Right
    //     -0.5, -0.5, -1.0, // Bottom Left
    //     0.0, 0.5, -1.0, // Top
    // ];

    // let verticies = [
    //     // counter-clock-wise (default with face culling)
    //     // triangle fan
    //     // x, y, z
    //     -0.5f32, 0.5, 1.0, // top left
    //     -0.5, -0.5, 1.0, // bottom left
    //     0.5, -0.5, 1.0, // bottom right
    //     0.5, 0.5, 1.0, // top right
    // ];

    // let verticies = [
    //     // ccw tri strip
    //     // x, y, z
    //     1f32, 1.0, 1.0, // right top front
    //     1.0, 1.0, -1.0, // right top back
    //     1.0, -1.0, 1.0, // right bottom front
    //     1.0, -1.0, -1.0, // right bottom back
    //     -1.0, 1.0, 1.0, // left top front
    //     -1.0, 1.0, -1.0, // left top back
    //     -1.0, -1.0, 1.0, // left bottom front
    //     -1.0, -1.0, -1.0, // left bottom back
    // ];

    // let verticies = [
    //     // x, y, z
    //     // front face
    //     1f32, 1.0, 1.0, // right top front
    //     -1.0, 1.0, 1.0, // left top front
    //     -1.0, -1.0, 1.0, // left bottom front
    //     1.0, -1.0, 1.0, // right bottom front
    //     // right face
    //     // in 1.0, -1.0, 1.0, // right bottom front
    //     1.0, -1.0, -1.0, // right bottom back
    //     1.0, 1.0, -1.0, // right top back
    //     1.0, 1.0, 1.0, // right top front
    //     // up face
    //     // in 1.0, 1.0, 1.0, // right top front
    //     1.0, 1.0, -1.0, // right top back
    //     -1.0, 1.0, -1.0, // left top back
    //     -1.0, 1.0, 1.0, // left top front
    //     // left face
    //     // in -1.0, 1.0, 1.0, // left top front
    //     -1.0, 1.0, -1.0, // left top back
    //     -1.0, -1.0, -1.0, // left bottom back
    //     -1.0, -1.0, 1.0, // left bottom front
    //     // down face
    //     // in -1.0, -1.0, 1.0, // left bottom front
    //     -1.0, -1.0, -1.0, // left bottom back
    //     1.0, -1.0, -1.0, // right bottom back
    //     1.0, -1.0, 1.0, // right bottom front
    //     // back face
    //     // in 1.0, -1.0, 1.0, // right bottom front
    //     1.0, 1.0, -1.0, // right top back
    //     -1.0, 1.0, -1.0, // left top back
    //     -1.0, -1.0, -1.0, // left bottom back
    //     1.0, -1.0, -1.0, // right bottom back
    // ];

    let faces = [
        [
            // front
            1f32, 1.0, 1.0, // right top front
            -1.0, 1.0, 1.0, // left top front
            -1.0, -1.0, 1.0, // left bottom front
            1.0, -1.0, 1.0, // right bottom front
        ],
        [
            // right
            1.0, -1.0, 1.0, // right bottom front
            1.0, -1.0, -1.0, // right bottom back
            1.0, 1.0, -1.0, // right top back
            1.0, 1.0, 1.0, // right top front
        ],
        [
            // up
            1.0, 1.0, 1.0, // right top front
            1.0, 1.0, -1.0, // right top back
            -1.0, 1.0, -1.0, // left top back
            -1.0, 1.0, 1.0, // left top front
        ],
        [
            // left face
            -1.0, 1.0, 1.0, // left top front
            -1.0, 1.0, -1.0, // left top back
            -1.0, -1.0, -1.0, // left bottom back
            -1.0, -1.0, 1.0, // left bottom front
        ],
        [
            // down face
            -1.0, -1.0, 1.0, // left bottom front
            -1.0, -1.0, -1.0, // left bottom back
            1.0, -1.0, -1.0, // right bottom back
            1.0, -1.0, 1.0, // right bottom front
        ],
        [
            // back face
            1.0, 1.0, -1.0, // right top back
            -1.0, 1.0, -1.0, // left top back
            -1.0, -1.0, -1.0, // left bottom back
            1.0, -1.0, -1.0, // right bottom back
        ],
    ];

    // let verticies = [
    //     // clock-wise
    //     // triangle fan
    //     // x, y, z
    //     -0.5f32, 0.5, 1.0, // top left
    //     0.5, 0.5, 1.0, // top right
    //     0.5, -0.5, 1.0, // bottom right
    //     -0.5, -0.5, -1.0, // bottom left
    // ];
    //
    // let verticies = [
    //     // don't do this
    //     // for consecutive draw
    //     // x, y, z
    //     -0.5f32, -0.5, -1.0, // bottom left
    //     -0.5, 0.5, 1.0, // top left
    //     0.5, -0.5, 1.0, // bottom right
    //     0.5, 0.5, 1.0, // top right
    // ];
    // let verticies = [
    //     // x, y, z
    //     -1f32, -1.0, -1.0, // bottom left
    //     -1.0, 1.0, 1.0, // top left
    //     1.0, -1.0, 1.0, // bottom right
    //     1.0, 1.0, 1.0, // top right
    // ];

    let verticies = faces.flatten();

    let buffer = gl.create_buffer().expect("Cannot create buffer");
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
    gl.buffer_data(glow::ARRAY_BUFFER, &verticies, glow::STATIC_DRAW);
    for i in 0..6 {
        gl.buffer_data(glow::ARRAY_BUFFER, &faces[i], glow::STATIC_DRAW);
    }
    let vertex_array = gl
        .create_vertex_array()
        .expect("Cannot create vertex array");
    gl.bind_vertex_array(Some(vertex_array));
    gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, (3 * size_of::<f32>()) as i32, 0);
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(
        1,
        3,
        glow::FLOAT,
        false,
        (3 * size_of::<f32>()) as i32,
        (3 * size_of::<f32>()) as i32,
    );

    gl.enable(glow::CULL_FACE);

    'run: loop {
        for event in event_loop.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'run;
                },
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(new_width, new_height) => {
                        debug!("Window: Resizing to {}x{}", new_width, new_height);
                        gl.viewport(0, 0, new_width, new_height);
                    },
                    a => {
                        debug!("Window: {a:?}")
                    },
                },
                Event::KeyDown {
                    scancode: Some(scancode),
                    keymod,
                    ..
                } => {
                    debug!("KeyDown:");
                    match scancode {
                        Scancode::C if keymod.contains(Mod::LCTRLMOD) => {
                            error!("^C");
                            break 'run;
                        },
                        Scancode::Up => {},
                        Scancode::Down => {},
                        Scancode::Left => {},
                        Scancode::Right => {},
                        Scancode::Space => {
                            // do step in game
                        },
                        _ => {},
                    }
                },
                Event::KeyUp { .. } => debug!("KeyUp"),
                Event::MouseButtonDown { .. } => debug!("MouseButtonDown"),
                Event::MouseButtonUp { .. } => debug!("MouseButtonUp"),
                a => debug!("WTF: {a:?}"),
            }
        }
        gl.clear_color(0.1, 0.2, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);

        gl.use_program(Some(program));
        gl.bind_vertex_array(Some(vertex_array));
        gl.draw_arrays(glow::POINTS, 0, 12);
        //, (verticies.len() / 3usize) as i32);
        // wireframe
        // gl.draw_arrays(glow::LINE_LOOP, 0, 8);
        // gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 12);
        // gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);
        gl.draw_arrays(glow::TRIANGLES, 0, 12);

        gl.bind_vertex_array(None);

        window.gl_swap_window();
    }

    gl.delete_vertex_array(vertex_array);
    gl.delete_buffer(buffer);
    gl.delete_program(program);
}

unsafe fn compile_shaders(
    gl: &Context,
    vertex_sources: Vec<String>,
    fragment_sources: Vec<String>,
) -> Vec<NativeShader> {
    let shaders = &[
        (glow::VERTEX_SHADER, vertex_sources),
        (glow::FRAGMENT_SHADER, fragment_sources),
    ]
    .iter()
    .flat_map(|(shader_type, sources)| {
        sources.iter().map(|source| {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                error!(
                    "Cannot create {} shader: {}",
                    match *shader_type {
                        glow::VERTEX_SHADER => "vertex",
                        glow::FRAGMENT_SHADER => "fragment",
                        _ => "<unknown>",
                    },
                    gl.get_shader_info_log(shader)
                );
            }
            shader
        })
    })
    .collect::<Vec<_>>();
    shaders.to_owned()
}

unsafe fn link_shaders(gl: &Context, shaders: Vec<NativeShader>) -> Program {
    let program = gl.create_program().expect("Cannot create program");
    shaders
        .iter()
        .for_each(|&shader| gl.attach_shader(program, shader));
    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        error!("Cannot link program: {}", gl.get_program_info_log(program));
    }
    // delete shaders after linking
    shaders.iter().for_each(|&shader| gl.delete_shader(shader));
    program
}
