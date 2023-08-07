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
    use glm::{vec3, vec4};
    let opaque_quads = &[
        // red quad
        (vec3(1., 0., 0.), ModelMatrixInput {position: vec3(0., 0., 0.), rotation: vec3(0., 0., 0.,), scale: vec3(1., 1., 1.)}),
        (vec3(0.5, 0.5, 0.), ModelMatrixInput {position: vec3(1., 0., 1.), rotation: vec3(0., 0., 0.), scale: vec3(1., 1., 1.)}),
    ];
    let tr_quads = &[
        (vec4(0., 1., 0., 0.5), ModelMatrixInput {position: vec3(0., 0., 2.), rotation: vec3(0., 0., 0.), scale: vec3(1., 1., 1.)}),
        (vec4(0.5, 0., 0.5, 0.5), ModelMatrixInput {position: vec3(0., 0., 3.), rotation: vec3(0., 0., 0.), scale: vec3(1., 2., 1.)}),
        (vec4(1., 1., 1., 0.5), ModelMatrixInput {position: vec3(0., 0., 4.), rotation: vec3(0., 0., 0.), scale: vec3(1., 1., 1.)}),
    ];
    unsafe { main1(opaque_quads, tr_quads) }
}

unsafe fn main1(opq: &[(glm::Vec3, ModelMatrixInput)], trp: &[(glm::Vec4, ModelMatrixInput)]) {
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

    macro_rules! glerr {
        () => {{
            let err = gl.raw().get_error();
            if err != 0 {
                eprintln!("line {} error {}", line!(), err);
                return;
            }
        }};
    }
    let mouse = sdl.mouse();
    mouse.set_relative_mouse_mode(true);
    
    gl.raw().enable(glow::DEBUG_OUTPUT);
    gl.raw().debug_message_callback(debug_message_callback);

    struct Shaders<'a> {
        solid: GLProgram<'a>,
        transparent: GLProgram<'a>,
        composite: GLProgram<'a>,
        screen: GLProgram<'a>,
    }
    let shaders = {
        let solid_shaders = &[
            (GLShaderType::Vertex, "./data/shaders/solid_v.glsl"),
            (GLShaderType::Fragment, "./data/shaders/solid_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let transparent_shaders = &[
            (GLShaderType::Vertex, "./data/shaders/transparent_v.glsl"),
            (GLShaderType::Fragment, "./data/shaders/transparent_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let composite_shaders = &[
            (GLShaderType::Vertex, "./data/shaders/composite_v.glsl"),
            (GLShaderType::Fragment, "./data/shaders/composite_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let screen_shaders = &[
            (GLShaderType::Vertex, "./data/shaders/screen_v.glsl"),
            (GLShaderType::Fragment, "./data/shaders/screen_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));

        let solid = load_shaders(&gl, solid_shaders);
        let transparent = load_shaders(&gl, transparent_shaders);
        let composite = load_shaders(&gl, composite_shaders);
        let screen = load_shaders(&gl, screen_shaders);
        Shaders {
            solid,
            transparent,
            composite,
            screen,
        }
    };

    let quad_vertices: &[f32] = &[
		// positions     uv
		-1.0, -1.0, 0.0, 0.0, 0.0,
		 1.0, -1.0, 0.0, 1.0, 0.0,
		 1.0,  1.0, 0.0, 1.0, 1.0,

		 1.0,  1.0, 0.0, 1.0, 1.0,
		-1.0,  1.0, 0.0, 0.0, 1.0,
		-1.0, -1.0, 0.0, 0.0, 0.0
    ];
    const F32S: i32 = std::mem::size_of::<f32>() as i32;
    let quad_vao = gl.raw().create_vertex_array().unwrap();
    let quad_vbo = gl.raw().create_buffer().unwrap();
    gl.raw().bind_vertex_array(Some(quad_vao));
    gl.raw().bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
    gl.raw().buffer_data_u8_slice(glow::ARRAY_BUFFER, memcast::as_bytes(quad_vertices), glow::STATIC_DRAW);
    gl.raw().enable_vertex_attrib_array(0);
    gl.raw().vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 5 * F32S, 0);
    gl.raw().enable_vertex_attrib_array(1);
    gl.raw().vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 5 * F32S, 3 * F32S);
    gl.raw().bind_vertex_array(None);

    // set up framebuffers and their texture attachments
    let opaque_fbo = gl.raw().create_framebuffer().unwrap();
    let transparent_fbo = gl.raw().create_framebuffer().unwrap();
    // attachments opaque
    let opaque_tx = gl.raw().create_texture().unwrap();
    gl.raw().bind_texture(glow::TEXTURE_2D, Some(opaque_tx));
    gl.raw().tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA16F as i32, width as i32, height as i32, 0, glow::RGBA, glow::HALF_FLOAT, None);
    gl.raw().tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
    gl.raw().tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
    gl.raw().bind_texture(glow::TEXTURE_2D, None);

    let depth_tx = gl.raw().create_texture().unwrap();
    gl.raw().bind_texture(glow::TEXTURE_2D, Some(depth_tx));
    gl.raw().tex_image_2d(glow::TEXTURE_2D, 0, glow::DEPTH_COMPONENT as i32, width as i32, height as i32, 0, glow::DEPTH_COMPONENT, glow::FLOAT, None);
    gl.raw().bind_texture(glow::TEXTURE_2D, None);

    gl.raw().bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));
    gl.raw().framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(opaque_tx), 0);
    gl.raw().framebuffer_texture_2d(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, glow::TEXTURE_2D, Some(depth_tx), 0);
    if gl.raw().check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        eprintln!("Framebuffer error: line {}", line!());
    }
    glerr!();
    gl.raw().bind_framebuffer(glow::FRAMEBUFFER, None);
    // attachments transparent
    let accum_tx = gl.raw().create_texture().unwrap();
    gl.raw().bind_texture(glow::TEXTURE_2D, Some(accum_tx));
    gl.raw().tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA16F as i32, width as i32, height as i32, 0, glow::RGBA, glow::HALF_FLOAT, None);
    gl.raw().tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
    gl.raw().tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
    gl.raw().bind_texture(glow::TEXTURE_2D, None);
    
    let reveal_tx = gl.raw().create_texture().unwrap();
    gl.raw().bind_texture(glow::TEXTURE_2D, Some(reveal_tx));
    gl.raw().tex_image_2d(glow::TEXTURE_2D, 0, glow::R8 as i32, width as i32, height as i32, 0, glow::RED, glow::FLOAT, None);
    gl.raw().tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
    gl.raw().tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
    gl.raw().bind_texture(glow::TEXTURE_2D, None);
    
    gl.raw().bind_framebuffer(glow::FRAMEBUFFER, Some(transparent_fbo));
    gl.raw().framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(accum_tx), 0);
    gl.raw().framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT1, glow::TEXTURE_2D, Some(reveal_tx), 0);
    gl.raw().framebuffer_texture_2d(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, glow::TEXTURE_2D, Some(depth_tx), 0); // from opaque
    gl.raw().draw_buffers(&[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1]);
    if gl.raw().check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        eprintln!("Framebuffer error: line {}", line!());
    }
    glerr!();
    gl.raw().bind_framebuffer(glow::FRAMEBUFFER, None);
    
    // transform matrices
    let opaque_objs = opq.iter().map(|(c, mmi)| (c, model_mat_from(*mmi))).collect::<Vec<_>>();
    let transparent_objs = trp.iter().map(|(c, mmi)| (c, model_mat_from(*mmi))).collect::<Vec<_>>();

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
    let solid_mvp_u = shaders.solid.get_uniform::<glm::Mat4>("mvp");
    let solid_color_u = shaders.solid.get_uniform::<glm::Vec3>("color");
    let tr_mvp_u = shaders.transparent.get_uniform::<glm::Mat4>("mvp");
    let tr_color_u = shaders.transparent.get_uniform::<glm::Vec4>("color");

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
            gl.raw().enable(glow::DEPTH_TEST);
            gl.raw().depth_func(glow::LESS);
            gl.raw().depth_mask(true);
            gl.raw().disable(glow::BLEND);
            gl.raw().clear_color(0.1, 0.2, 0.3, 0.);
            // bind opaque buffer
            gl.raw().bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));
            gl.raw().clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.set_program(&shaders.solid);

            for (&c, mm) in &opaque_objs {
                solid_mvp_u.set(vp_mat * *mm, false);
                solid_color_u.set(c);
                gl.raw().bind_vertex_array(Some(quad_vao));
                gl.raw().draw_arrays(glow::TRIANGLES, 0, 6);
                draw_calls += 1;
            }
        }
        // transparent
        {
            gl.raw().depth_mask(false);
            gl.raw().enable(glow::BLEND);
            gl.raw().blend_func_draw_buffer(0, glow::ONE, glow::ONE);
            gl.raw().blend_func_draw_buffer(1, glow::ZERO, glow::ONE_MINUS_SRC_COLOR);
            gl.raw().blend_equation(glow::FUNC_ADD);

            gl.raw().bind_framebuffer(glow::FRAMEBUFFER, Some(transparent_fbo));
            gl.raw().clear_buffer_f32_slice(glow::COLOR, 0, &[0., 0., 0., 0.]);
            gl.raw().clear_buffer_f32_slice(glow::COLOR, 1, &[1., 1., 1., 1.]);

            gl.set_program(&shaders.transparent);

            for (&c, mm) in &transparent_objs {
                tr_mvp_u.set(vp_mat * *mm, false);
                tr_color_u.set(c);
                gl.raw().bind_vertex_array(Some(quad_vao));
                gl.raw().draw_arrays(glow::TRIANGLES, 0, 6);
                draw_calls += 1;
            }
        }
        // composite
        {
            gl.raw().depth_func(glow::ALWAYS);
            gl.raw().enable(glow::BLEND);
            gl.raw().blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            gl.raw().bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));

            gl.set_program(&shaders.composite);
            // draw screen quad
            gl.raw().active_texture(glow::TEXTURE0);
            gl.raw().bind_texture(glow::TEXTURE_2D, Some(accum_tx));
            gl.raw().active_texture(glow::TEXTURE1);
            gl.raw().bind_texture(glow::TEXTURE_2D, Some(reveal_tx));
            gl.raw().bind_vertex_array(Some(quad_vao));
            gl.raw().draw_arrays(glow::TRIANGLES, 0, 6);
            draw_calls += 1;
        }
        // backbuffer
        {
            gl.raw().disable(glow::DEPTH_TEST);
            gl.raw().depth_mask(true); // enable depth mask to later clear depth buffer
            gl.raw().disable(glow::BLEND);

            // unbind framebuffer == now render to screen backbuffer
            gl.raw().bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.raw().clear_color(0., 0., 0., 0.);
            gl.raw().clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);

            gl.set_program(&shaders.screen);
            // draw final screen quad
            gl.raw().active_texture(glow::TEXTURE0);
            gl.raw().bind_texture(glow::TEXTURE_2D, Some(opaque_tx));
            gl.raw().bind_vertex_array(Some(quad_vao));
            gl.raw().draw_arrays(glow::TRIANGLES, 0, 6);
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
}

#[derive(Clone, Copy)]
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
    draw_depth: bool,
}

fn debug_message_callback(_source: u32, _typ: u32, id: u32, _severity: u32, message: &str) {
    eprintln!("GL error {:0x}: {}", id, message);
}

#[cfg(xxx)]
unsafe fn main0() {
    // let args: Vec<String> = std::env::args().collect();
    let (width, height): (u32, u32) = (800, 600);
    let (width2, height2) = (width, height);
    // let (width2, height2) = (width.next_power_of_two(), height.next_power_of_two());
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

    macro_rules! glerr {
        () => {{
            let err = gl.raw().get_error();
            if err != 0 {
                eprintln!("line {} error {}", line!(), err);
                return;
            }
        }};
    }

    let mouse = sdl.mouse();

    gl.raw().enable(glow::DEBUG_OUTPUT);
    gl.raw().debug_message_callback(debug_message_callback);

    // shaders
    /*let shaders_raw = &[
        (GLShaderType::Vertex, "./data/shaders/vertex.glsl"),
        (GLShaderType::Fragment, "./data/shaders/fragment.glsl"),
    ]
    .map(|(t, p)| (t, std::path::Path::new(p)));
    let program = load_shaders(&gl, shaders_raw);*/
    // NOTE 1: compile shaders for different types of objects
    struct Shaders<'a> {
        solid: GLProgram<'a>,
        transparent: GLProgram<'a>,
        composite: GLProgram<'a>,
        screen: GLProgram<'a>,
    }
    let shaders = {
        let solid_shaders = &[
            (GLShaderType::Vertex, "./data/shaders/solid_v.glsl"),
            (GLShaderType::Fragment, "./data/shaders/solid_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let transparent_shaders = &[
            (GLShaderType::Vertex, "./data/shaders/transparent_v.glsl"),
            (GLShaderType::Fragment, "./data/shaders/transparent_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let composite_shaders = &[
            (GLShaderType::Vertex, "./data/shaders/composite_v.glsl"),
            (GLShaderType::Fragment, "./data/shaders/composite_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));
        let screen_shaders = &[
            (GLShaderType::Vertex, "./data/shaders/screen_v.glsl"),
            (GLShaderType::Fragment, "./data/shaders/screen_f.glsl"),
        ]
        .map(|(t, p)| (t, std::path::Path::new(p)));

        
        let solid = load_shaders(&gl, solid_shaders);
        let transparent = load_shaders(&gl, transparent_shaders);
        let composite = load_shaders(&gl, composite_shaders);
        let screen = load_shaders(&gl, screen_shaders);
        Shaders {
            solid,
            transparent,
            composite,
            screen,
        }
    };

    // init some gl things
    // gl.raw().clear_color(0.1, 0.2, 0.3, 1.0);
    // NOTE 2: create vertex array and buffers
    let vao = gl.raw().create_vertex_array().unwrap();
    let elements = gl.raw().create_buffer().unwrap();
    let vbo = gl
        .get_vertex_attribute(0, GLBufferTarget::Array, 3, GLType::Float)
        .unwrap();
    let uv_buf = gl
        .get_vertex_attribute(1, GLBufferTarget::Array, 2, GLType::Float)
        .unwrap();
    let norm_buf = gl
        .get_vertex_attribute(2, GLBufferTarget::Array, 3, GLType::Float)
        .unwrap();
    let screen_quad_vao = gl.raw().create_vertex_array().unwrap();
    let screen_quad_vbo = gl.raw().create_buffer().unwrap();
    let screen_quad_uv = gl.raw().create_buffer().unwrap();
    /*let mvp_u = program.get_uniform::<glm::Mat4>("MVP");
    let m_u = program.get_uniform::<glm::Mat4>("M");
    let v_u = program.get_uniform::<glm::Mat4>("V");
    let light_pos_w_u = program.get_uniform::<glm::Vec3>("lightPosition_w");
    let light_power_u = program.get_uniform::<f32>("lightPower");
    let light_intensity_u = program.get_uniform::<glm::IVec3>("lightIntensity");
    let time_u = program.get_uniform::<f32>("time");
    let sampler_u = program.get_uniform::<i32>("sampler");
    let draw_depth_u = program.get_uniform::<i32>("drawDepth");
    let near_u = program.get_uniform::<f32>("near");
    let far_u = program.get_uniform::<f32>("far");*/

    // generate meshes and sort them
    // TODO: atlases and batching
    // NOTE 3: load objects and materials
    /*let (models, materials) = {
        let (models, materials) = load_obj(
            [
                "./data/objects/sample.obj",
                "./data/objects/sample_2.obj",
                "./data/objects/dice.obj",
                "./data/objects/box.obj",
                "./data/objects/wb_oit_test.obj",
            ][4],
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        )
        .unwrap();
        let materials = materials.unwrap();
        (models, materials)
    };*/

    // NOTE 4: load textures
    /*let mut textures = load_textures(&gl, &materials);
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
    */
    // NOTE 5: models are ordered by material_id
    // for now hardcode material 0 as opaque (simple)
    // and materials 1 and 2 as transparent (complex)
    // it will be baked.offsets[0], 1 and 2 respectively
    // let simple_mats = [0];
    // let complex_mats = [1, 2];
    /*let simple_mats = materials
        .iter()
        .enumerate()
        .filter(|(_, m)| m.name == "a")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let complex_mats = materials
        .iter()
        .enumerate()
        .filter(|(_, m)| m.name != "a")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let baked = bake_meshes(models, &simple_mats);
    let screen_pos: &[f32] = &[0., 0., 1., 0., 1., 1., 1., 1., 0., 1., 0., 0.];
    let screen_uv: &[f32] = &[-1., -1., 1., -1., 1., 1., 1., 1., -1., 1., -1., -1.];
    {
        // NOTE: block with gl data sending
        gl.bind_vertex_array(vao);
        gl.write_to_buffer(
            GLBufferTarget::ElementArray,
            elements,
            &baked.indices,
            GLBufferUsage::StaticDraw,
        );
        vbo.write(&baked.vertices, GLBufferUsage::StaticDraw);
        uv_buf.write(&baked.uvs, GLBufferUsage::StaticDraw);
        norm_buf.write(&baked.normals, GLBufferUsage::StaticDraw);
        gl.bind_vertex_array(screen_quad_vao);
        gl.write_to_buffer(
            GLBufferTarget::Array,
            screen_quad_vbo,
            screen_pos,
            GLBufferUsage::StaticDraw,
        );
        gl.write_to_buffer(
            GLBufferTarget::Array,
            screen_quad_uv,
            screen_uv,
            GLBufferUsage::StaticDraw,
        );
        gl.bind_buffer(GLBufferTarget::Array, screen_quad_vbo);
        gl.raw()
            .vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
        gl.bind_buffer(GLBufferTarget::Array, screen_quad_uv);
        gl.raw()
            .vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
    }*/
    // NOTE 6: framebuffers
    let opaque_fbo = gl.raw().create_framebuffer().unwrap();
    let transparent_fbo = gl.raw().create_framebuffer().unwrap();

    // simple
    let opaque_tx = gl
        .create_texture(GLTextureTarget::Texture2D, GLColor::Rgba16f)
        .unwrap();
    opaque_tx.bind();
    opaque_tx.clear(0, width2, height2, GLColor::Rgba, GLType::HalfFloat);
    opaque_tx.min_filter(GLTextureMinFilter::Nearest);
    opaque_tx.mag_filter(GLTextureMagFilter::Nearest);

    let depth_tx = gl
        .create_texture(GLTextureTarget::Texture2D, GLColor::DepthComponent)
        .unwrap();
    depth_tx.clear(0, width2, height2, GLColor::DepthComponent, GLType::Float);
    glerr!();
    gl.raw()
        .bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));
    gl.raw().framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D,
        Some(opaque_tx.raw()),
        0,
    );
    gl.raw().framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::DEPTH_ATTACHMENT,
        glow::TEXTURE_2D,
        Some(depth_tx.raw()),
        0,
    );
    glerr!();
    if gl.raw().check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        println!("Framebuffer error");
    }
    // -- simple
    // complex
    let accum_tx = gl
        .create_texture(GLTextureTarget::Texture2D, GLColor::Rgba16f)
        .unwrap();
    let reveal_tx = gl
        .create_texture(GLTextureTarget::Texture2D, GLColor::R8)
        .unwrap();
    accum_tx.bind();
    accum_tx.clear(0, width2, height2, GLColor::Rgba, GLType::HalfFloat);
    accum_tx.min_filter(GLTextureMinFilter::Nearest);
    accum_tx.mag_filter(GLTextureMagFilter::Nearest);
    glerr!();

    reveal_tx.bind();
    reveal_tx.clear(0, width2, height2, GLColor::Red, GLType::Float);
    reveal_tx.min_filter(GLTextureMinFilter::Nearest);
    reveal_tx.mag_filter(GLTextureMagFilter::Nearest);
    glerr!();
    gl.raw()
        .bind_framebuffer(glow::FRAMEBUFFER, Some(transparent_fbo));
    gl.raw().framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D,
        Some(accum_tx.raw()),
        0,
    );
    gl.raw().framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT1,
        glow::TEXTURE_2D,
        Some(reveal_tx.raw()),
        0,
    );
    gl.raw().framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::DEPTH_ATTACHMENT,
        glow::TEXTURE_2D,
        Some(depth_tx.raw()),
        0,
    );
    glerr!();

    let transparent_draw_buffers = &[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1];
    gl.raw().draw_buffers(transparent_draw_buffers);
    if gl.raw().check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        println!("Framebuffer error (transparent)");
    }
    // -- comlex
    gl.raw().bind_framebuffer(glow::FRAMEBUFFER, None);
    // -- note 6

    /*
    gl.raw().enable(glow::CULL_FACE);
    gl.raw().enable(glow::DEPTH_TEST);
    gl.raw().depth_func(glow::LESS);
    gl.raw().enable(glow::BLEND);
    gl.raw()
        .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);*/

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

    state.mouse.set_relative_mouse_mode(true);

    let mut prev_time = 0.0;
    let mut current_time;
    let mut delta_time;
    let mut draw_calls: u32;
    let light_position = glm::vec3(4., 3., 3.);

    let solid_mvp_u = shaders.solid.get_uniform::<glm::Mat4>("MVP");
    let solid_m_u = shaders.solid.get_uniform::<glm::Mat4>("M");
    let solid_v_u = shaders.solid.get_uniform::<glm::Mat4>("V");
    let solid_light_pos_w_u = shaders.solid.get_uniform::<glm::Vec3>("lightPosition_w");
    let solid_light_power_u = shaders.solid.get_uniform::<f32>("lightPower");
    let solid_light_intensity_u = shaders.solid.get_uniform::<glm::IVec3>("lightIntensity");
    let solid_draw_depth_u = shaders.solid.get_uniform::<i32>("drawDepth");
    let solid_near_u = shaders.solid.get_uniform::<f32>("near");
    let solid_far_u = shaders.solid.get_uniform::<f32>("far");
    let solid_sampler_u = shaders.solid.get_uniform::<i32>("sampler");

    let transparent_mvp_u = shaders.transparent.get_uniform::<glm::Mat4>("MVP");
    let transparent_m_u = shaders.transparent.get_uniform::<glm::Mat4>("M");
    let transparent_v_u = shaders.transparent.get_uniform::<glm::Mat4>("V");
    let transparent_light_pos_w_u = shaders
        .transparent
        .get_uniform::<glm::Vec3>("lightPosition_w");
    let transparent_light_power_u = shaders.transparent.get_uniform::<f32>("lightPower");
    let transparent_light_intensity_u = shaders
        .transparent
        .get_uniform::<glm::IVec3>("lightIntensity");
    let transparent_draw_depth_u = shaders.transparent.get_uniform::<i32>("drawDepth");
    let transparent_near_u = shaders.transparent.get_uniform::<f32>("near");
    let transparent_far_u = shaders.transparent.get_uniform::<f32>("far");
    let transparent_sampler_u = shaders.transparent.get_uniform::<i32>("sampler");

    // let screen_sampler_u = shaders.screen.get_uniform::<i32>("screen");

    'render: loop {
        let (z_near, z_far) = (0.1, 100.0);
        let ComputedMatrices {
            mvp: mvp_mat,
            model: model_mat,
            view: view_mat,
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
        draw_calls = 0;
        // NOTE 7: draw solid objects
        gl.raw().enable(glow::CULL_FACE);
        gl.raw().enable(glow::DEPTH_TEST);
        gl.raw().depth_func(glow::LESS);
        gl.raw().depth_mask(true);
        gl.raw().disable(glow::BLEND);
        gl.raw().clear_color(0., 0., 0., 0.);
        // bind opaque framebuffer
        glerr!();
        gl.raw()
            .bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));
        glerr!();
        gl.raw()
            .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        glerr!();
        // use solid shader
        gl.set_program(&shaders.solid);
        glerr!();
        solid_mvp_u.set(model_mat, false);
        solid_m_u.set(model_mat, false);
        solid_v_u.set(view_mat, false);
        solid_light_pos_w_u.set(light_position);
        solid_light_power_u.set(state.light_power);
        solid_light_intensity_u.set(state.light_intensity);
        solid_draw_depth_u.set(state.draw_depth as i32);
        solid_near_u.set(z_near);
        solid_far_u.set(z_far);

        glerr!();
        gl.bind_vertex_array(vao);
        glerr!();
        vbo.enable(false, 0, 0);
        uv_buf.enable(false, 0, 0);
        norm_buf.enable(false, 0, 0);
        glerr!();
        for &i in &simple_mats {
            solid_sampler_u.set(i as i32);
            gl.raw().active_texture(glow::TEXTURE0 + i as u32);
            glerr!();
            gl.raw().draw_elements(
                glow::TRIANGLES,
                baked.lengths[i],
                glow::UNSIGNED_INT,
                baked.offsets[i],
            );
            glerr!();
            draw_calls += 1;
        }
        vbo.disable();
        uv_buf.disable();
        norm_buf.disable();
        // -- note 7
        // NOTE 8: draw transparent objects
        glerr!();
        gl.raw().depth_mask(false);
        gl.raw().enable(glow::BLEND);
        gl.raw().blend_func_draw_buffer(0, glow::ONE, glow::ONE);
        gl.raw()
            .blend_func_draw_buffer(1, glow::ZERO, glow::ONE_MINUS_SRC_COLOR);
        gl.raw().blend_equation(glow::FUNC_ADD);
        glerr!();
        // bind transparent framebuffer
        gl.raw()
            .bind_framebuffer(glow::FRAMEBUFFER, Some(transparent_fbo));
        glerr!();
        gl.raw()
            .clear_buffer_f32_slice(glow::COLOR, 0, &[0., 0., 0., 0.]);
        glerr!();
        gl.raw()
            .clear_buffer_f32_slice(glow::COLOR, 1, &[1., 1., 1., 1.]);
        glerr!();
        // use transparent shader
        gl.set_program(&shaders.transparent);
        glerr!();
        transparent_mvp_u.set(mvp_mat, false);
        transparent_m_u.set(model_mat, false);
        transparent_v_u.set(view_mat, false);
        transparent_light_pos_w_u.set(light_position);
        transparent_light_power_u.set(state.light_power);
        transparent_light_intensity_u.set(state.light_intensity);
        transparent_draw_depth_u.set(state.draw_depth as i32);
        transparent_near_u.set(z_near);
        transparent_far_u.set(z_far);

        glerr!();
        gl.bind_vertex_array(vao);
        glerr!();
        vbo.enable(false, 0, 0);
        uv_buf.enable(false, 0, 0);
        norm_buf.enable(false, 0, 0);
        glerr!();
        for &i in &complex_mats {
            transparent_sampler_u.set(i as i32);
            gl.raw().active_texture(glow::TEXTURE0 + i as u32);
            glerr!();
            gl.raw().draw_elements(
                glow::TRIANGLES,
                baked.lengths[i],
                glow::UNSIGNED_INT,
                baked.offsets[i],
            );
            glerr!();
            draw_calls += 1;
        }
        vbo.disable();
        uv_buf.disable();
        norm_buf.disable();
        glerr!();
        // -- note 8
        // NOTE 9: draw composite image
        gl.raw().depth_func(glow::ALWAYS);
        gl.raw().enable(glow::BLEND);
        gl.raw()
            .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        glerr!();
        // bind opaque framebuffer
        gl.raw()
            .bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));
        glerr!();
        gl.set_program(&shaders.composite);
        gl.raw().active_texture(glow::TEXTURE0);
        accum_tx.bind();
        gl.raw().active_texture(glow::TEXTURE1);
        reveal_tx.bind();
        glerr!();
        gl.bind_vertex_array(screen_quad_vao);
        glerr!();
        gl.raw().draw_arrays(glow::TRIANGLES, 0, 6);
        glerr!();
        draw_calls += 1;
        // -- note 9
        // NOTE 10: draw to backbuffer
        gl.raw().disable(glow::DEPTH_TEST);
        gl.raw().depth_mask(true); // to clear depth buffer with glClear
        gl.raw().disable(glow::BLEND);
        glerr!();
        // bind backbuffer
        gl.raw().bind_framebuffer(glow::FRAMEBUFFER, None);
        glerr!();
        gl.raw().clear_color(0., 0., 0., 0.);
        gl.raw()
            .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        glerr!();
        // screen shader
        gl.set_program(&shaders.screen);
        glerr!();
        // screen_sampler_u.set(0);
        gl.raw().active_texture(glow::TEXTURE0);
        opaque_tx.bind();
        glerr!();
        gl.bind_vertex_array(screen_quad_vao);
        glerr!();
        gl.raw().draw_arrays(glow::TRIANGLES, 0, 6);
        glerr!();
        draw_calls += 1;
        // -- note 10
        // NOTE 11
        gl.raw().bind_framebuffer(glow::FRAMEBUFFER, None);
        glerr!();
        gl.raw().clear_color(1., 1., 1., 1.);
        gl.raw()
            .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        glerr!();
        // -- note 11

        /*
        gl.raw()
            .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

        /*gl.set_program(&program);
        // pass data to shaders
        mvp_u.set(mvp_mat, false);
        m_u.set(model_mat, false);
        v_u.set(view_mat, false);
        light_pos_w_u.set(glm::vec3(4., 3., 3.));
        light_power_u.set(state.light_power);
        light_intensity_u.set(state.light_intensity);
        time_u.set(start.elapsed().unwrap().as_secs_f32());
        draw_depth_u.set(state.draw_depth as i32);
        near_u.set(z_near);
        far_u.set(z_far);*/

        // enable buffers
        vbo.enable(false, 0, 0);
        uv_buf.enable(false, 0, 0);
        norm_buf.enable(false, 0, 0);

        gl.bind_buffer(GLBufferTarget::ElementArray, elements);
        // NOTE: max simultaneous textures is 32
        for (i, tx) in textures.iter() {
            let i = *i;
            // sampler_u.set(i as i32);
            gl.raw().active_texture(glow::TEXTURE0 + i as u32);
            tx.bind();
            gl.raw().draw_elements(
                glow::TRIANGLES,
                baked.lengths[i],
                glow::UNSIGNED_INT,
                baked.offsets[i],
            );
            draw_calls += 1;
        }
        // blanks is skipped for now
        // finish drawing*/
        state.window.gl_swap_window();
        // vbo.disable();
        // uv_buf.disable();
        // norm_buf.disable();
        
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
                    unsafe {
                        if state.culling {
                            state.gl.raw().enable(glow::CULL_FACE)
                        } else {
                            state.gl.raw().disable(glow::CULL_FACE)
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
