use glmc::*;
use glow::HasContext;
use tobj::load_obj;

fn main() {
    unsafe { main0() }
}

struct GameState {
    position: glm::Vec3,
    rotation: glm::Vec2,
    mouse: sdl2::mouse::MouseUtil,
    window: sdl2::video::Window,
    captured: bool,
}

unsafe fn main0() {
    // let args: Vec<String> = std::env::args().collect();
    let (width, height): (u32, u32) = (800, 600);
    let start = std::time::SystemTime::now();
    let aspect_ratio = width as f32 / height as f32;
    let fov = glm::radians(45.);
    // sdl2 and gl context
    let (gl, window, mut event_loop, _gl_context) =
        init_window(width, height).expect("Cannot initialize window");
    let mouse = window.subsystem().sdl().mouse();
    // shaders
    let shaders_raw = &[
        (glow::VERTEX_SHADER, "./data/shaders/vertex.glsl"),
        (glow::FRAGMENT_SHADER, "./data/shaders/fragment.glsl"),
    ]
    .map(|(t, p)| (t, std::path::Path::new(p)));
    let program = load_shaders(&gl, shaders_raw);

    // init some gl things
    gl.clear_color(0.1, 0.2, 0.3, 1.0);
    let vao = gl.create_vertex_array().unwrap();
    let vbo = gl.create_buffer().unwrap();
    let uv_buf = gl.create_buffer().unwrap();
    let mvp_u = gl.get_uniform_location(program, "MVP");
    let time_u = gl.get_uniform_location(program, "time");
    let sampler_u = gl.get_uniform_location(program, "sampler");

    // send data to buffer
    let (cube, materials) = {
        let (models, materials) =
            load_obj("./data/objects/cube.obj", &tobj::LoadOptions::default()).unwrap();
        let materials = materials.unwrap();
        (models, materials)
    };
    let cube = &cube[0].mesh;
    let texture = &materials[0].diffuse_texture;

    let cube_texture = {
        if let Some(tx_path) = texture {
            let tx = Some(gl.create_texture().unwrap());
            gl.bind_texture(glow::TEXTURE_2D, tx);
            let txr_img = image::load(
                std::io::BufReader::new(std::fs::File::open(tx_path).unwrap()),
                image::ImageFormat::Png,
            )
            .unwrap()
            .into_rgb8();
            let txr_data = txr_img.as_flat_samples().samples;
            let (w, h) = (txr_img.width() as i32, txr_img.height() as i32);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                w,
                h,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(txr_data),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            tx
        } else {
            None
        }
    };
    let cube_triangles: &[f32] = &cube
        .indices
        .iter()
        .map(|i| *i as usize)
        .flat_map(|i| [3 * i, 3 * i + 1, 3 * i + 2])
        .map(|i| cube.positions[i])
        //.flat_map(
        //    |i| [cube.positions[3*i], cube.positions[3*i+1], cube.positions[3*i+2]]
        //)
        .collect::<Vec<_>>();
    println!("cube.indices {:?} -- {}", cube.indices, cube.indices.len());
    println!(
        "cube.positions {:?} -- {}",
        cube.positions,
        cube.positions.len()
    );
    println!(
        "cube_triangles {:?} -- {}",
        cube_triangles,
        cube_triangles.len()
    );
    let cube_uv: &[f32] = &cube
        .texcoord_indices
        .iter()
        .map(|&i| i as usize)
        .flat_map(
            // NOTE: u, 1-v opengl-tutorial says it's DirectX format, but it also works
            // with blender cube, so assume all models are in this format
            |i| [cube.texcoords[2 * i], 1.0 - cube.texcoords[2 * i + 1]],
        )
        .collect::<Vec<_>>();
    println!(
        "cube.texcoords {:?} -- {}",
        cube.texcoords,
        cube.texcoords.len()
    );
    println!(
        "cube.texcoord_indices {:?} -- {}",
        cube.texcoord_indices,
        cube.texcoord_indices.len()
    );
    println!("cube_uv {:?} -- {}", cube_uv, cube_uv.len());
    gl.bind_vertex_array(Some(vao));

    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        slice_as_u8(cube_triangles),
        glow::STATIC_DRAW,
    );
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(uv_buf));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, slice_as_u8(cube_uv), glow::STATIC_DRAW);
    // NOTE
    gl.enable(glow::CULL_FACE);
    // gl.enable(glow::DEPTH_TEST);
    gl.depth_func(glow::LESS);

    let mut state = GameState {
        position: glm::vec3(0., 0., -4.5),
        rotation: glm::vec2(0., 0.),
        // NOTE
        mouse,
        window,
        captured: true,
    };

    state.mouse.set_relative_mouse_mode(true);

    let mut prev_time = 0.0;
    let mut current_time;
    let mut delta_time;

    let mut wasd = glm::ivec3(0, 0, 0);

    'render: loop {
        let (mvp_mat, right, front) = {
            let z_near = 0.1;
            let z_far = 100.0;
            let projection = glm::ext::perspective(fov, aspect_ratio, z_near, z_far);

            let (cx, sx) = (glm::cos(state.rotation.x), glm::sin(state.rotation.x));
            let (cy, sy) = (glm::cos(state.rotation.y), glm::sin(state.rotation.y));
            let direction = glm::vec3(cy * sx, sy, cy * cx);
            let right_angle = state.rotation.x - std::f32::consts::FRAC_PI_2;
            let right = glm::vec3(glm::sin(right_angle), 0.0, glm::cos(right_angle));
            let up = glm::cross(right, direction);
            let front = -glm::cross(right, glm::vec3(0.0, 1.0, 0.0));

            let view = glm::ext::look_at(state.position, state.position + direction, up);
            let model = MAT4_ONE;
            (projection * view * model, right, front)
        };

        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

        gl.use_program(Some(program));
        // pass data to shaders
        gl.uniform_matrix_4_f32_slice(mvp_u.as_ref(), false, mat4_as_vec(mvp_mat));
        gl.uniform_1_f32(time_u.as_ref(), start.elapsed().unwrap().as_secs_f32());

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, cube_texture);
        gl.uniform_1_i32(sampler_u.as_ref(), 0);
        // enable buffers
        gl.enable_vertex_attrib_array(0);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(uv_buf));
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
        // draw and release
        gl.draw_arrays(glow::TRIANGLES, 0, (cube_triangles.len() / 3) as i32);
        state.window.gl_swap_window();
        gl.disable_vertex_attrib_array(0);
        gl.disable_vertex_attrib_array(1);

        let mouse_speed: f32 = 0.005;
        let speed: f32 = 2.0;
        current_time = start.elapsed().unwrap().as_secs_f32();
        delta_time = current_time - prev_time;

        for event in event_loop.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Scancode;
            match event {
                Event::Quit { .. } => break 'render,
                Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => {
                    let (w, h) = state.window.size();
                    let (w2, h2) = (w as i32 / 2, h as i32 / 2);
                    state.mouse.warp_mouse_in_window(&state.window, w2, h2);
                    state.captured = !state.captured;
                    state.mouse.set_relative_mouse_mode(state.captured);
                },
                Event::MouseMotion { xrel, yrel, .. } if state.captured => {
                    // xrel + == right
                    // yrel + == down
                    state.rotation.x -= mouse_speed * xrel as f32;
                    state.rotation.y -= mouse_speed * yrel as f32;
                },
                Event::KeyDown {
                    scancode: Some(scancode),
                    repeat: false,
                    ..
                } if state.captured => {
                    /*println!("{:?}", event);
                    let position_diff = match scancode {
                        Scancode::W => front,
                        Scancode::A => -right,
                        Scancode::S => -front,
                        Scancode::D => right,
                        Scancode::Space => VEC3_UP,
                        Scancode::LShift => -VEC3_UP,
                        _ => continue
                    };
                    state.position = state.position + position_diff * delta_time * speed;*/
                    match scancode {
                        // x+ forward
                        // y+ up
                        // z+ right
                        Scancode::W => wasd.x += 1,
                        Scancode::A => wasd.z -= 1,
                        Scancode::S => wasd.x -= 1,
                        Scancode::D => wasd.z += 1,
                        Scancode::Space => wasd.y += 1,
                        Scancode::LShift => wasd.y -= 1,
                        _ => handle_event(event, &mut state),
                    }
                },
                Event::KeyUp {
                    scancode: Some(scancode),
                    repeat: false,
                    ..
                } if state.captured => {
                    // println!("{:?}", event);
                    // inverted KeyDowm
                    match scancode {
                        Scancode::W => wasd.x -= 1,
                        Scancode::A => wasd.z += 1,
                        Scancode::S => wasd.x += 1,
                        Scancode::D => wasd.z -= 1,
                        Scancode::Space => wasd.y -= 1,
                        Scancode::LShift => wasd.y += 1,
                        _ => handle_event(event, &mut state),
                    }
                },
                _ => handle_event(event, &mut state),
            }
        }
        let pos_diff = front * wasd.x as f32 + right * wasd.z as f32 + VEC3_UP * wasd.y as f32;
        state.position = state.position + pos_diff * delta_time * speed;
        prev_time = current_time;
    }
}

fn handle_event(event: sdl2::event::Event, state: &mut GameState) {
    use sdl2::event::Event;
    use sdl2::keyboard::Scancode;

    match event {
        Event::MouseWheel {
            ..
            // timestamp, window_id, which, x, y, direction
        } => {
            println!("{:?}", event);
        },
        Event::KeyDown {
            scancode: Some(Scancode::P), ..
        } => {
            println!("Position: {:?}", state.position);
        },
        Event::KeyDown {
            // timestamp, window_id, keycode,
            // scancode, //, keymod
            repeat: false,
            ..
        } => {
            // println!("{:?}", event);
        },
        _ => {}
    }
}

type GlowShaderType = u32;

unsafe fn load_shaders(
    gl: &glow::Context,
    shaders: &[(GlowShaderType, &std::path::Path)],
) -> glow::NativeProgram {
    let program = gl.create_program().expect("Cannot create program");

    let mut shaders_compiled = Vec::with_capacity(shaders.len());
    for (shader_type, path) in shaders {
        let path_abs = path
            .canonicalize()
            .unwrap_or_else(|_| panic!("Cannot load shader: {}", path.display()));
        let source = std::fs::read_to_string(&path_abs).unwrap();
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, &source);

        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{} {}", &path_abs.display(), gl.get_shader_info_log(shader));
        }

        gl.attach_shader(program, shader);
        shaders_compiled.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders_compiled {
        gl.detach_shader(program, shader);
    }
    program
}

unsafe fn init_window(
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
    // init attrs
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_flags().forward_compatible().set();
    // create window
    let window = video
        .window("", width, height)
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let gl_context = window.gl_create_context()?;
    let gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
    let event_loop = sdl.event_pump()?;
    Ok((gl, window, event_loop, gl_context))
}

unsafe fn slice_as_u8<T>(data: &[T]) -> &[u8] {
    std::slice::from_raw_parts(
        data.as_ptr() as *const u8,
        data.len() * core::mem::size_of::<T>(),
    )
}

unsafe fn mat4_as_vec(a: glm::Mat4) -> &'static [f32] {
    std::slice::from_raw_parts(a.as_array().as_ptr() as *const f32, 16)
}

#[allow(dead_code)]
mod glmc {
    use glm::{Mat4, Vec3, Vec4};
    #[macro_export]
    macro_rules! vec3 {
        ($x: expr, $y: expr, $z: expr) => {
            Vec3 {
                x: $x,
                y: $y,
                z: $z,
            }
        };
    }
    #[macro_export]
    macro_rules! vec4 {
        ($x: expr, $y: expr, $z: expr, $w: expr) => {
            Vec4 {
                x: $x,
                y: $y,
                z: $z,
                w: $w,
            }
        };
    }
    #[macro_export]
    macro_rules! mat4 {
        ($c0: expr, $c1: expr, $c2: expr, $c3: expr) => {
            Mat4 {
                c0: $c0,
                c1: $c1,
                c2: $c2,
                c3: $c3,
            }
        };
    }
    pub const VEC3_UP: Vec3 = vec3!(0., 1., 0.);
    pub const VEC4_ZERO: Vec4 = vec4!(0., 0., 0., 0.);
    pub const MAT4_ZERO: Mat4 = mat4!(VEC4_ZERO, VEC4_ZERO, VEC4_ZERO, VEC4_ZERO);
    pub const MAT4_ONE: Mat4 = mat4!(
        vec4!(1., 0., 0., 0.),
        vec4!(0., 1., 0., 0.),
        vec4!(0., 0., 1., 0.),
        vec4!(0., 0., 0., 1.)
    );
}
