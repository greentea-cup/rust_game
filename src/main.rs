mod glmc;
mod loader;
mod memcast;
use crate::glmc::*;
use crate::loader::*;
use crate::memcast::*;
use glow::HasContext;
use tobj::load_obj;

fn main() {
    unsafe { main0() }
}

struct GameState {
    position: glm::Vec3,
    rotation: glm::Vec2,
    light_power: f32,
    captured: bool,
}

unsafe fn main0() {
    // let args: Vec<String> = std::env::args().collect();
    let (width, height): (u32, u32) = (800, 600);
    let start = std::time::SystemTime::now();
    let aspect_ratio = width as f32 / height as f32;
    let fov = glm::radians(45.);
    // sdl2 and gl context
    let InitializedWindow {
        gl,
        sdl,
        window,
        mut event_loop,
        #[allow(unused)]
        gl_context, // holds gl context, should not be dropped
    } = init_window(width, height).expect("Cannot initialize window");
    let mouse = sdl.mouse();
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
    let norm_buf = gl.create_buffer().unwrap();
    let mvp_u = gl.get_uniform_location(program, "MVP");
    let m_u = gl.get_uniform_location(program, "M");
    let v_u = gl.get_uniform_location(program, "V");
    let light_pos_w_u = gl.get_uniform_location(program, "lightPosition_w");
    let light_power_u = gl.get_uniform_location(program, "lightPower");
    let time_u = gl.get_uniform_location(program, "time");
    let sampler_u = gl.get_uniform_location(program, "sampler");

    // generate meshes and sort them
    // TODO: atlases and batching
    let (models, materials) = {
        let (models, materials) = load_obj(
            "./data/objects/sample.obj",
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        )
        .unwrap();
        let materials = materials.unwrap();
        (models, materials)
    };

    let textures = load_textures(&gl, &materials);
    let baked = bake_meshes(models);

    gl.bind_vertex_array(Some(vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        slice_as_u8(&baked.vertices),
        glow::STATIC_DRAW,
    );
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(uv_buf));
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        slice_as_u8(&baked.uvs),
        glow::STATIC_DRAW,
    );
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(norm_buf));
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        slice_as_u8(&baked.normals),
        glow::STATIC_DRAW,
    );
    // NOTE
    let mut culling = true;
    gl.enable(glow::CULL_FACE);
    gl.enable(glow::DEPTH_TEST);
    gl.depth_func(glow::LESS);

    let mut state = GameState {
        position: glm::vec3(9., 3.7, 1.25),
        rotation: glm::vec2(-1.7, -0.4),
        light_power: 50.0,
        captured: true,
    };

    mouse.set_relative_mouse_mode(true);

    let mut prev_time = 0.0;
    let mut current_time;
    let mut delta_time;

    let mut wasd = glm::ivec3(0, 0, 0);
    let mut fast = false;

    'render: loop {
        let ComputedMatrices {
            mvp: mvp_mat,
            model: model_mat,
            view: view_mat,
            right,
            front,
        } = compute_matrices(state.position, state.rotation, fov, aspect_ratio);

        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

        gl.use_program(Some(program));
        // pass data to shaders
        gl.uniform_matrix_4_f32_slice(mvp_u.as_ref(), false, mat4_as_vec(mvp_mat));
        gl.uniform_matrix_4_f32_slice(m_u.as_ref(), false, mat4_as_vec(model_mat));
        gl.uniform_matrix_4_f32_slice(v_u.as_ref(), false, mat4_as_vec(view_mat));
        gl.uniform_3_f32(light_pos_w_u.as_ref(), 4., 3., 3.);
        gl.uniform_1_f32(light_power_u.as_ref(), state.light_power);
        gl.uniform_1_f32(time_u.as_ref(), start.elapsed().unwrap().as_secs_f32());
        // enable buffers
        gl.enable_vertex_attrib_array(0);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(uv_buf));
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(2);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(norm_buf));
        gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, 0, 0);

        // NOTE: max simultaneous textures is 32
        for (i, &tx) in textures.iter().enumerate() {
            gl.uniform_1_i32(sampler_u.as_ref(), i as i32);
            gl.active_texture(glow::TEXTURE0 + i as u32);
            gl.bind_texture(glow::TEXTURE_2D, tx);
            gl.draw_arrays(glow::TRIANGLES, baked.offsets[i], baked.lengths[i]);
        }
        // blanks is skipped for now
        // finish drawing
        window.gl_swap_window();
        gl.disable_vertex_attrib_array(0);
        gl.disable_vertex_attrib_array(1);
        gl.disable_vertex_attrib_array(2);

        let mouse_speed: f32 = 0.005;
        let speed_fast: f32 = 5.0;
        let speed_slow: f32 = 2.0;
        let speed = if fast { speed_fast } else { speed_slow };
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
                    wasd.x = 0;
                    wasd.y = 0;
                    wasd.z = 0;
                    fast = false;
                    let (w, h) = window.size();
                    let (w2, h2) = (w as i32 / 2, h as i32 / 2);
                    mouse.warp_mouse_in_window(&window, w2, h2);
                    state.captured = !state.captured;
                    mouse.set_relative_mouse_mode(state.captured);
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
                        Scancode::Tab => fast = true,
                        Scancode::G => {
                            culling = !culling;
                            if culling {
                                gl.enable(glow::CULL_FACE);
                                println!("Culling on");
                            } else {
                                gl.disable(glow::CULL_FACE);
                                println!("Culling off");
                            }
                        },
                        _ => handle_event(event, &mut state),
                    }
                },
                Event::KeyUp {
                    scancode: Some(scancode),
                    repeat: false,
                    ..
                } if state.captured => {
                    match scancode {
                        // inverted KeyDown
                        Scancode::W => wasd.x -= 1,
                        Scancode::A => wasd.z += 1,
                        Scancode::S => wasd.x += 1,
                        Scancode::D => wasd.z -= 1,
                        Scancode::Space => wasd.y -= 1,
                        Scancode::LShift => wasd.y += 1,
                        Scancode::Tab => fast = false,
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
        Event::KeyDown {
            scancode: Some(Scancode::R),
            ..
        } => {
            let glm::Vec2 { x, y } = state.rotation;
            println!("Rotation: {} {}", x, y);
        },
        Event::KeyDown {
            scancode: Some(Scancode::P),
            ..
        } => {
            let glm::Vec3 { x, y, z } = state.position;
            println!("Position: {} {} {}", x, y, z);
        },
        Event::KeyDown {
            scancode: Some(Scancode::L),
            ..
        } => {
            println!("Light power: {}", state.light_power);
        },
        Event::MouseWheel { y, .. } if state.captured => {
            state.light_power += 5.0 * y as f32;
            state.light_power = state.light_power.clamp(0.0, 100.0);
        },
        _ => {},
    }
}
