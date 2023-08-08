mod gl_utils;
mod glmc;
mod loader;
mod memcast;
use crate::glmc::*;
use crate::loader::*;
use glow::HasContext;
use tobj::load_obj;

fn main() {
    use glm::{vec3, vec4};
    let opaque_quads = &[
        // red quad
        (
            vec3(1., 0., 0.),
            ModelMatrixInput {
                position: vec3(0., 0., 0.),
                rotation: vec3(0., 0., 0.),
                scale: vec3(1., 1., 1.),
            },
        ),
        (
            vec3(0.5, 0.5, 0.),
            ModelMatrixInput {
                position: vec3(1., 0., 1.),
                rotation: vec3(0., 0., 0.),
                scale: vec3(1., 1., 1.),
            },
        ),
    ];
    let tr_quads = &[
        (
            vec4(0., 1., 0., 0.5),
            ModelMatrixInput {
                position: vec3(0., 0., 2.),
                rotation: vec3(0., 0., 0.),
                scale: vec3(1., 1., 1.),
            },
        ),
        (
            vec4(0.5, 0., 0.5, 0.5),
            ModelMatrixInput {
                position: vec3(0., 0., 3.),
                rotation: vec3(0., 0., 0.),
                scale: vec3(1., 2., 1.),
            },
        ),
        (
            vec4(1., 1., 1., 0.5),
            ModelMatrixInput {
                position: vec3(0., 0., 4.),
                rotation: vec3(0., 0., 0.),
                scale: vec3(1., 1., 1.),
            },
        ),
    ];
    unsafe { main0(opaque_quads, tr_quads).unwrap() }
}

unsafe fn main0(opaque: &[(glm::Vec3, ModelMatrixInput)], transparent: &[(glm::Vec4, ModelMatrixInput)]) -> Result<(), String> {
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
        gl_context,
    } = init_window(width, height)?;

    macro_rules! glerr {
        () => {{
            let err = gl.get_error();
            if err != 0 {
                eprintln!("line {} error {}", line!(), err);
                return Err(format!("GL Error {} on line {}", err, line!()));
            }
        }};
    }
    let mouse = sdl.mouse();
    mouse.set_relative_mouse_mode(true);

    gl.enable(glow::DEBUG_OUTPUT);
    gl.debug_message_callback(debug_message_callback);

    struct Shaders {
        solid: glow::Program,
        transparent: glow::Program,
        composite: glow::Program,
        screen: glow::Program,
    }
    let shaders = {
        let solid_shaders = &[
            (glow::VERTEX_SHADER, "./data/shaders/solid_v.glsl"),
            (glow::FRAGMENT_SHADER, "./data/shaders/solid_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let transparent_shaders = &[
            (glow::VERTEX_SHADER, "./data/shaders/transparent_v.glsl"),
            (glow::FRAGMENT_SHADER, "./data/shaders/transparent_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let composite_shaders = &[
            (glow::VERTEX_SHADER, "./data/shaders/composite_v.glsl"),
            (glow::FRAGMENT_SHADER, "./data/shaders/composite_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let screen_shaders = &[
            (glow::VERTEX_SHADER, "./data/shaders/screen_v.glsl"),
            (glow::FRAGMENT_SHADER, "./data/shaders/screen_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));

        let solid = load_shaders(&gl, solid_shaders)?;
        let transparent = load_shaders(&gl, transparent_shaders)?;
        let composite = load_shaders(&gl, composite_shaders)?;
        let screen = load_shaders(&gl, screen_shaders)?;
        Shaders {
            solid,
            transparent,
            composite,
            screen,
        }
    };

    let quad_vertices: &[f32] = &[
        // x, y, z, u, v
        -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
        1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0,
    ];
    const F32S: i32 = std::mem::size_of::<f32>() as i32;
    let quad_vao = gl.create_vertex_array()?;
    let quad_vbo = gl.create_buffer()?;
    gl.bind_vertex_array(Some(quad_vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        memcast::as_bytes(quad_vertices),
        glow::STATIC_DRAW,
    );
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 5 * F32S, 0);
    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 5 * F32S, 3 * F32S);
    gl.bind_vertex_array(None);

    // set up framebuffers and their texture attachments
    let opaque_fbo = gl.create_framebuffer()?;
    let transparent_fbo = gl.create_framebuffer()?;
    // attachments opaque
    let opaque_tx = gl.create_texture()?;
    gl.bind_texture(glow::TEXTURE_2D, Some(opaque_tx));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA16F as i32,
        width as i32,
        height as i32,
        0,
        glow::RGBA,
        glow::HALF_FLOAT,
        None,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.bind_texture(glow::TEXTURE_2D, None);

    let depth_tx = gl.create_texture()?;
    gl.bind_texture(glow::TEXTURE_2D, Some(depth_tx));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::DEPTH_COMPONENT as i32,
        width as i32,
        height as i32,
        0,
        glow::DEPTH_COMPONENT,
        glow::FLOAT,
        None,
    );
    gl.bind_texture(glow::TEXTURE_2D, None);

    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D,
        Some(opaque_tx),
        0,
    );
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::DEPTH_ATTACHMENT,
        glow::TEXTURE_2D,
        Some(depth_tx),
        0,
    );
    if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        eprintln!("Framebuffer error: line {}", line!());
    }
    glerr!();
    gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    // attachments transparent
    let accum_tx = gl.create_texture()?;
    gl.bind_texture(glow::TEXTURE_2D, Some(accum_tx));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA16F as i32,
        width as i32,
        height as i32,
        0,
        glow::RGBA,
        glow::HALF_FLOAT,
        None,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.bind_texture(glow::TEXTURE_2D, None);

    let reveal_tx = gl.create_texture()?;
    gl.bind_texture(glow::TEXTURE_2D, Some(reveal_tx));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::R8 as i32,
        width as i32,
        height as i32,
        0,
        glow::RED,
        glow::FLOAT,
        None,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.bind_texture(glow::TEXTURE_2D, None);

    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(transparent_fbo));
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D,
        Some(accum_tx),
        0,
    );
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT1,
        glow::TEXTURE_2D,
        Some(reveal_tx),
        0,
    );
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::DEPTH_ATTACHMENT,
        glow::TEXTURE_2D,
        Some(depth_tx),
        0,
    ); // from opaque
    gl.draw_buffers(&[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1]);
    if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        eprintln!("Framebuffer error: line {}", line!());
    }
    glerr!();
    gl.bind_framebuffer(glow::FRAMEBUFFER, None);

    // transform matrices
    let opaque_objs = opaque
        .iter()
        .map(|(c, mmi)| (c, model_mat_from(*mmi)))
        .collect::<Vec<_>>();
    let transparent_objs = transparent
        .iter()
        .map(|(c, mmi)| (c, model_mat_from(*mmi)))
        .collect::<Vec<_>>();

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
        draw_depth: false,
    };
    let mut prev_time = 0.0;
    let mut current_time;
    let mut delta_time;
    let mut draw_calls: u32;
    let light_position = glm::vec3(4., 3., 3.);

    // uniforms
    let solid_mvp_u = gl.get_uniform_location(shaders.solid, "mvp");
    let solid_color_u = gl.get_uniform_location(shaders.solid, "color");
    let tr_mvp_u = gl.get_uniform_location(shaders.solid, "mvp");
    let tr_color_u = gl.get_uniform_location(shaders.solid, "color");

    'render: loop {
        let (z_near, z_far) = (0.1, 100.0);
        let ComputedMatrices {
            view: view_mat,
            projection: proj_mat,
            right,
            front,
        } = compute_matrices(
            state.position,
            state.rotation,
            fov,
            aspect_ratio,
            z_near,
            z_far,
        );
        let vp_mat = proj_mat * view_mat;
        draw_calls = 0;

        // render
        // solid
        {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.depth_mask(true);
            gl.disable(glow::BLEND);
            gl.clear_color(0.1, 0.2, 0.3, 0.);
            // bind opaque buffer
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(shaders.solid));

            for (&c, mm) in &opaque_objs {
                gl.uniform_matrix_4_f32_slice(
                    solid_mvp_u.as_ref(),
                    false,
                    &memcast::mat4_as_array(vp_mat * *mm),
                );
                gl.uniform_3_f32_slice(solid_color_u.as_ref(), c.as_array());
                gl.bind_vertex_array(Some(quad_vao));
                gl.draw_arrays(glow::TRIANGLES, 0, 6);
                draw_calls += 1;
            }
        }
        // transparent
        {
            gl.depth_mask(false);
            gl.enable(glow::BLEND);
            gl.blend_func_draw_buffer(0, glow::ONE, glow::ONE);
            gl.blend_func_draw_buffer(1, glow::ZERO, glow::ONE_MINUS_SRC_COLOR);
            gl.blend_equation(glow::FUNC_ADD);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(transparent_fbo));
            gl.clear_buffer_f32_slice(glow::COLOR, 0, &[0., 0., 0., 0.]);
            gl.clear_buffer_f32_slice(glow::COLOR, 1, &[1., 1., 1., 1.]);

            gl.use_program(Some(shaders.transparent));

            for (&c, mm) in &transparent_objs {
                gl.uniform_matrix_4_f32_slice(
                    tr_mvp_u.as_ref(),
                    false,
                    &memcast::mat4_as_array(vp_mat * *mm),
                );
                gl.uniform_4_f32_slice(tr_color_u.as_ref(), c.as_array());
                gl.bind_vertex_array(Some(quad_vao));
                gl.draw_arrays(glow::TRIANGLES, 0, 6);
                draw_calls += 1;
            }
        }
        // composite
        {
            gl.depth_func(glow::ALWAYS);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));

            gl.use_program(Some(shaders.composite));
            // draw screen quad
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(accum_tx));
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(reveal_tx));
            gl.bind_vertex_array(Some(quad_vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            draw_calls += 1;
        }
        // backbuffer
        {
            gl.disable(glow::DEPTH_TEST);
            gl.depth_mask(true); // enable depth mask to later clear depth buffer
            gl.disable(glow::BLEND);

            // unbind framebuffer == now render to screen backbuffer
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.clear_color(0., 0., 0., 0.);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);

            gl.use_program(Some(shaders.screen));
            // draw final screen quad
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(opaque_tx));
            gl.bind_vertex_array(Some(quad_vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            draw_calls += 1;
        }

        state.window.gl_swap_window();
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
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct ModelMatrixInput {
    position: glm::Vec3,
    rotation: glm::Vec3,
    scale: glm::Vec3,
}

fn model_mat_from(i: ModelMatrixInput) -> glm::Mat4 {
    let mut res = glmc::MAT4_ONE;
    res = glm::ext::translate(&res, i.position);
    res = glm::ext::rotate(&res, glm::radians(i.rotation.x), glm::vec3(1., 0., 0.));
    res = glm::ext::rotate(&res, glm::radians(i.rotation.y), glm::vec3(0., 1., 0.));
    res = glm::ext::rotate(&res, glm::radians(i.rotation.z), glm::vec3(0., 0., 1.));
    res = glm::ext::scale(&res, i.scale);
    res
}

struct GameState<'a> {
    gl: &'a glow::Context,
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
    draw_depth: bool,
}

fn debug_message_callback(_source: u32, _typ: u32, id: u32, _severity: u32, message: &str) {
    eprintln!("GL error {:0x}: {}", id, message);
}

/*
 let mut textures = load_textures(&gl, &materials);
    if false {
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
            // NOTE rgb = 3, rgba = 4, grayscale = 1
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
        let ttx = gl
            .create_texture(GLTextureTarget::Texture2D, GLColor::Rgba)
            .unwrap();
        ttx.bind();
        ttx.write(
            0,
            w as u32,
            h as u32,
            GLColor::Rgb,
            GLType::UnsignedByte,
            &text_texture,
        );
        ttx.mag_filter(GLTextureMagFilter::Nearest);
        ttx.min_filter(GLTextureMinFilter::Nearest);
        textures.insert(text_material_id, ttx);
    }
:*/

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
                    unsafe {
                        if state.culling {
                            state.gl.enable(glow::CULL_FACE)
                        } else {
                            state.gl.disable(glow::CULL_FACE)
                        }
                    }
                },
                Scancode::T => intensity!(x),
                Scancode::Y => intensity!(y),
                Scancode::U => intensity!(z),
                Scancode::N => state.draw_depth = !state.draw_depth,
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
                let glm::IVec3 { x, y, z } = state.light_intensity;
                println!("Light A{} D{} S{}", x, y, z);
            }
            println!("Draw depth: {}", state.draw_depth);
            println!("Draw calls: {}", state.draw_calls);
        },
        Event::MouseWheel { y, .. } if state.captured => {
            state.light_power += 5.0 * y as f32;
            state.light_power = state.light_power.clamp(0.0, 100.0);
        },
        _ => {},
    }
}
