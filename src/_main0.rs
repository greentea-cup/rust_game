use glow::*;

fn main() {
    unsafe { main0() }
}

unsafe fn main0() {
    let (width, height): (u32, u32) = (800, 600);

    // Create a context from an sdl2 window

    let (gl, mut window, mut events_loop, _context) =
        create_sdl2_context(width, height).expect("Error: cannot setup sdl and/or gl env");

    // Create a shader program from source
    let (vertex_shader_source, fragment_shader_source) = {
        let v = std::fs::read_to_string("./data/shaders/vertex.glsl")
            .expect("Error: vertex shader cannot be loaded");
        let f = std::fs::read_to_string("./data/shaders/fragment.glsl")
            .expect("Error: fragment shader cannot be loaded");
        (v, f)
    };
    let program = create_program(&gl, &vertex_shader_source, &fragment_shader_source);
    gl.use_program(Some(program));

    // Create a vertex buffer and vertex array object
    let data = &[
        /*
        0.0f32, 1.0, 0.0, //
        -1.0, -1.0, 0.0, //
        1.0, -1.0, 0.0, //
        */
        -1.0f32, 1.0, 0.0, //
        -1.0, -1.0, 0.0, //
        1.0, -1.0, 0.0, //
        1.0, 1.0, 0.0, //
    ];
    let (vbo, vao) = create_vertex_buffer(&gl, data);

    // Upload some uniforms
    let start = std::time::SystemTime::now();
    let (mut graph_ox, mut graph_oy): (f32, f32) = (0.0, 0.0);
    let mut graph_scale = 1.0f32;
    let (mut width, mut height): (u32, u32) = window.size();
    // uniforms are ok to be in Option bc of glsl optimizations
    let u_secs = gl.get_uniform_location(program, "start_time");
    let u_graph = gl.get_uniform_location(program, "graph");
    let u_graph_scale = gl.get_uniform_location(program, "graph_scale");
    let u_window = gl.get_uniform_location(program, "window");
    gl.clear_color(0.1, 0.2, 0.3, 1.0);
    // gl.enable(glow::CULL_FACE);

    'render: loop {
        use sdl2::event::{Event, WindowEvent};
        use sdl2::keyboard::Scancode;
        use sdl2::mouse::MouseButton;
        use sdl2::video::FullscreenType;

        for event in events_loop.poll_iter() {
            match event {
                Event::Quit { .. } => break 'render,
                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {
                    gl.viewport(0, 0, x, y);
                    (width, height) = window.size();
                },
                Event::KeyUp {
                    scancode: Some(Scancode::F11),
                    ..
                } => window
                    .set_fullscreen(match window.fullscreen_state() {
                        FullscreenType::True => FullscreenType::Off,
                        _ => FullscreenType::True,
                    })
                    .unwrap(),
                Event::KeyUp {
                    scancode: Some(Scancode::F12),
                    ..
                } => window
                    .set_fullscreen(match window.fullscreen_state() {
                        FullscreenType::Desktop => FullscreenType::Off,
                        _ => FullscreenType::Desktop,
                    })
                    .unwrap(),
                Event::MouseMotion {
                    mousestate: m,
                    xrel,
                    yrel,
                    ..
                } => if m.left() {
                    graph_ox += -xrel as f32 / width as f32;
                    graph_oy += yrel as f32 / height as f32;
                    println!("x: {} y: {} scale: {}", graph_ox, graph_oy, graph_scale);
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
                    graph_ox = 0.0;
                    graph_oy = 0.0;
                    println!("x: {} y: {} scale: {}", graph_ox, graph_oy, graph_scale);
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Middle, ..} => {
                    graph_scale = 1.0;
                    println!("x: {} y: {} scale: {}", graph_ox, graph_oy, graph_scale);
                },
                Event::MouseWheel {
                    y, .. // ignore normal / flipped dir
                } => {
                    graph_scale += -0.1 * y as f32;
                    graph_scale = graph_scale.max(0.0);
                    println!("x: {} y: {} scale: {}", graph_ox, graph_oy, graph_scale);
                },
                _ => {},
            }
        }
        gl.uniform_1_f32(u_secs.as_ref(), start.elapsed().unwrap().as_secs_f32());
        gl.uniform_2_f32(u_graph.as_ref(), graph_ox, graph_oy);
        gl.uniform_1_f32(u_graph_scale.as_ref(), graph_scale);
        gl.uniform_2_u32(u_window.as_ref(), width, height);
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);
        window.gl_swap_window();
    }

    // Clean up
    gl.delete_program(program);
    gl.delete_vertex_array(vao);
    gl.delete_buffer(vbo)
}

unsafe fn create_sdl2_context(
    width: u32,
    height: u32,
) -> Result<
    (
        glow::Context,
        sdl2::video::Window,
        sdl2::EventPump,
        sdl2::video::GLContext,
    ),
    String,
> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_flags().forward_compatible().set();
    let window = match video.window("", width, height).opengl().resizable().build() {
        Err(error) => return Err(error.to_string()),
        Ok(window) => window,
    };
    let gl_context = window.gl_create_context()?;
    let gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
    let event_loop = sdl.event_pump()?;

    Ok((gl, window, event_loop, gl_context))
}

unsafe fn create_program(
    gl: &glow::Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> NativeProgram {
    let program = gl.create_program().expect("Cannot create program");

    let shader_sources = [
        (glow::VERTEX_SHADER, vertex_shader_source),
        (glow::FRAGMENT_SHADER, fragment_shader_source),
    ];

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}

unsafe fn create_vertex_buffer(
    gl: &glow::Context,
    data: &[f32],
) -> (NativeBuffer, NativeVertexArray) {
    // We construct a buffer and upload the data
    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * core::mem::size_of::<f32>(),
        ),
        glow::STATIC_DRAW,
    );

    // We now construct a vertex array to describe the format of the input buffer
    let vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vao));
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

    (vbo, vao)
}
