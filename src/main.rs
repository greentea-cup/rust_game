mod gl_wrapper;
mod glmc;
mod loader;
mod memcast;
use crate::gl_wrapper::*;
use crate::glmc::*;
use crate::loader::*;
use glow::HasContext;
use tobj::load_obj;

fn main() {
    unsafe { main0() }
}

struct GameState<'a> {
    gl: &'a GLWrapper,
    window: sdl2::video::Window,
    mouse: sdl2::mouse::MouseUtil,
    position: glm::Vec3,
    rotation: glm::Vec2,
    light_power: f32,
    light_intensity: glm::IVec3,
    captured: bool,
    wasd: glm::IVec3,
    fast: bool,
    running: bool,
    culling: bool,
    draw_calls: u32,
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
    } = init_window(width, height).expect("Cannot initialize window");
    let mouse = sdl.mouse();
    // shaders
    let shaders_raw = &[
        (GLShaderType::Vertex, "./data/shaders/vertex.glsl"),
        (GLShaderType::Fragment, "./data/shaders/fragment.glsl"),
    ]
    .map(|(t, p)| (t, std::path::Path::new(p)));
    let program = load_shaders(&gl, shaders_raw);

    // init some gl things
    gl.raw().clear_color(0.1, 0.2, 0.3, 1.0);
    let vao = gl.raw().create_vertex_array().unwrap();
    let vbo = gl
        .get_vertex_attribute(0, GLBufferTarget::Array, 3, GLType::Float)
        .unwrap();
    let uv_buf = gl
        .get_vertex_attribute(1, GLBufferTarget::Array, 2, GLType::Float)
        .unwrap();
    let norm_buf = gl
        .get_vertex_attribute(2, GLBufferTarget::Array, 3, GLType::Float)
        .unwrap();
    let mvp_u = program.get_uniform::<glm::Mat4>("MVP");
    let m_u = program.get_uniform::<glm::Mat4>("M");
    let v_u = program.get_uniform::<glm::Mat4>("V");
    let light_pos_w_u = program.get_uniform::<glm::Vec3>("lightPosition_w");
    let light_power_u = program.get_uniform::<f32>("lightPower");
    let light_intensity_u = program.get_uniform::<glm::IVec3>("lightIntensity");
    let time_u = program.get_uniform::<f32>("time");
    let sampler_u = program.get_uniform::<i32>("sampler");

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
    let mut textures = load_textures(&gl, &materials);
    {
        let text_material_id = materials
            .iter()
            .enumerate()
            .find(|(_, mtl)| mtl.name == "Text_material")
            .unwrap()
            .0;
        // NOTE: render sample text
        let fonts = &[fontdue::Font::from_bytes(
            std::fs::read("./data/fonts/DejaVuSansMono.ttf").unwrap(),
            fontdue::FontSettings::default(),
        )
        .unwrap()];
        let (text_texture, w, h) = {
            // NOTE rgb=3, rgba = 4, grayscale = 1
            let pixel_width = 3;
            use fontdue::layout::{CoordinateSystem, GlyphRasterConfig, Layout, TextStyle};
            let mut ly = Layout::new(CoordinateSystem::PositiveYDown);
            ly.append(fonts, &TextStyle::new("Test", 72.0, 0));
            ly.append(fonts, &TextStyle::new("\nsmaller\nlines\n", 40.0, 0));
            ly.append(fonts, &TextStyle::new("Русский", 40.0, 0));
            let (w0, h0) = {
                // TODO: h = ly.height(); w = /*compute width*/;
                let (mut x1, mut x2): (i32, i32) = (0, 0);
                for g in ly.glyphs() {
                    x1 = x1.min(g.x as i32);
                    x2 = x2.max(g.x as i32 + g.width as i32);
                }
                (1 + (x2 - x1) as usize, ly.height() as usize) // idk why 1+dx
            };

            let (w, h) = (
                w0.next_power_of_two().max(256),
                h0.next_power_of_two().max(256),
            );
            println!("{}x{} -> {}x{}", w0, h0, w, h);
            let mut res = vec![0; pixel_width * w * h];
            for g in ly.glyphs() {
                if g.width == 0 {
                    continue;
                }
                let GlyphRasterConfig {
                    glyph_index: g_index,
                    px: g_px,
                    ..
                } = g.key;
                let bmp = fonts[g.font_index].rasterize_indexed(g_index, g_px).1;
                let start = pixel_width * (g.y as usize * w + g.x as usize);
                for (i, row) in bmp.chunks(g.width).enumerate() {
                    for (j, &px) in row.iter().enumerate() {
                        let offset = start + pixel_width * (i * w + j);
                        // NOTE
                        // for i in 0..pixel_width {res[offset+i] = px;}
                        res[offset] = px;
                        res[offset + 1] = px;
                        res[offset + 2] = px;
                    }
                }
            }
            (res, w, h)
        };
        // TODO: adjust uvs for text plane(s)

        let ttx = Some(gl.raw().create_texture().unwrap());
        gl.raw().bind_texture(glow::TEXTURE_2D, ttx);
        gl.raw().tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGB as i32,
            w as i32,
            h as i32,
            0,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            Some(&text_texture),
        );
        gl.raw().tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        gl.raw().tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        textures[text_material_id] = ttx;
    }

    let baked = bake_meshes(models);
    {
        // NOTE: block with gl data sending
        gl.bind_vertex_array(vao);
        vbo.write(&gl, &baked.vertices, GLBufferUsage::StaticDraw);
        uv_buf.write(&gl, &baked.uvs, GLBufferUsage::StaticDraw);
        norm_buf.write(&gl, &baked.normals, GLBufferUsage::StaticDraw);
    }
    gl.raw().enable(glow::CULL_FACE);
    gl.raw().enable(glow::DEPTH_TEST);
    gl.raw().depth_func(glow::LESS);

    let mut state = GameState {
        gl: &gl,
        window,
        mouse,
        position: glm::vec3(5.2, 3.3, 0.),
        rotation: glm::vec2(-1.57, -1.),
        wasd: glm::ivec3(0, 0, 0),
        light_power: 50.0,
        light_intensity: glm::ivec3(1, 1, 1),
        captured: true,
        running: true,
        fast: false,
        culling: true,
        draw_calls: 0,
    };

    state.mouse.set_relative_mouse_mode(true);

    let mut prev_time = 0.0;
    let mut current_time;
    let mut delta_time;
    let mut draw_calls: u32;

    'render: loop {
        let ComputedMatrices {
            mvp: mvp_mat,
            model: model_mat,
            view: view_mat,
            right,
            front,
        } = compute_matrices(state.position, state.rotation, fov, aspect_ratio);

        gl.raw()
            .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

        gl.set_program(&program);
        // pass data to shaders
        mvp_u.set(mvp_mat, false);
        m_u.set(model_mat, false);
        v_u.set(view_mat, false);
        light_pos_w_u.set(glm::vec3(4., 3., 3.));
        light_power_u.set(state.light_power);
        light_intensity_u.set(state.light_intensity);
        time_u.set(start.elapsed().unwrap().as_secs_f32());

        // enable buffers
        vbo.enable(&gl, false, 0, 0);
        uv_buf.enable(&gl, false, 0, 0);
        norm_buf.enable(&gl, false, 0, 0);
        draw_calls = 0;
        // NOTE: max simultaneous textures is 32
        for (i, &tx) in textures.iter().enumerate() {
            sampler_u.set(i as i32);
            gl.raw().active_texture(glow::TEXTURE0 + i as u32);
            gl.raw().bind_texture(glow::TEXTURE_2D, tx);
            gl.raw()
                .draw_arrays(glow::TRIANGLES, baked.offsets[i], baked.lengths[i]);
            draw_calls += 1;
        }
        // blanks is skipped for now
        // finish drawing
        state.window.gl_swap_window();
        gl.raw().disable_vertex_attrib_array(0);
        gl.raw().disable_vertex_attrib_array(1);
        gl.raw().disable_vertex_attrib_array(2);
        state.draw_calls = draw_calls;
        let speed_fast: f32 = 5.0;
        let speed_slow: f32 = 2.0;
        let speed = if state.fast { speed_fast } else { speed_slow };
        current_time = start.elapsed().unwrap().as_secs_f32();
        delta_time = current_time - prev_time;

        for event in event_loop.poll_iter() {
            handle_event(event, &mut state);
            if !state.running {
                break 'render;
            }
        }
        let pos_diff = front * state.wasd.x as f32
            + right * state.wasd.z as f32
            + VEC3_UP * state.wasd.y as f32;
        state.position = state.position + pos_diff * delta_time * speed;
        prev_time = current_time;
    }
}

fn handle_event(event: sdl2::event::Event, state: &mut GameState) {
    use sdl2::event::Event;
    use sdl2::keyboard::Scancode;

    let mouse_speed: f32 = 0.005;

    match event {
        Event::Quit { .. } => {
            state.running = false;
        },
        Event::KeyDown {
            scancode: Some(Scancode::Escape),
            ..
        } => {
            state.wasd.x = 0;
            state.wasd.y = 0;
            state.wasd.z = 0;
            state.fast = false;
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
            macro_rules! intensity {
                ($field:ident) => {{
                    let i = state.light_intensity.$field;
                    state.light_intensity.$field = 1 - i;
                }};
            }
            match scancode {
                // x+ forward
                // y+ up
                // z+ right
                Scancode::W => state.wasd.x += 1,
                Scancode::A => state.wasd.z -= 1,
                Scancode::S => state.wasd.x -= 1,
                Scancode::D => state.wasd.z += 1,
                Scancode::Space => state.wasd.y += 1,
                Scancode::LShift => state.wasd.y -= 1,
                Scancode::Tab => state.fast = true,
                Scancode::G => {
                    state.culling = !state.culling;
                    if state.culling {
                        unsafe {
                            state.gl.raw().enable(glow::CULL_FACE);
                        }
                        println!("Culling on");
                    } else {
                        unsafe {
                            state.gl.raw().disable(glow::CULL_FACE);
                        }
                        println!("Culling off");
                    }
                },
                Scancode::T => intensity!(x),
                Scancode::Y => intensity!(y),
                Scancode::U => intensity!(z),
                _ => {},
            }
        },
        Event::KeyUp {
            scancode: Some(scancode),
            repeat: false,
            ..
        } if state.captured => {
            match scancode {
                // inverted KeyDown
                Scancode::W => state.wasd.x -= 1,
                Scancode::A => state.wasd.z += 1,
                Scancode::S => state.wasd.x += 1,
                Scancode::D => state.wasd.z -= 1,
                Scancode::Space => state.wasd.y -= 1,
                Scancode::LShift => state.wasd.y += 1,
                Scancode::Tab => state.fast = false,
                _ => {},
            }
        },
        Event::KeyDown {
            scancode: Some(Scancode::Q),
            ..
        } => {
            {
                let glm::Vec2 { x, y } = state.rotation;
                println!("Rotation: {} {}", x, y);
            }
            {
                let glm::Vec3 { x, y, z } = state.position;
                println!("Position: {} {} {}", x, y, z);
            }
            println!("Light power: {}", state.light_power);
            {
                let glm::IVec3 {x, y, z} = state.light_intensity;
                println!("Light A{} D{} S{}", x, y, z);
            }
            println!("Draw calls: {}", state.draw_calls);
        },
        Event::MouseWheel { y, .. } if state.captured => {
            state.light_power += 5.0 * y as f32;
            state.light_power = state.light_power.clamp(0.0, 100.0);
        },
        _ => {},
    }
}
