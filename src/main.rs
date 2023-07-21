use glmc::*;
use glow::HasContext;
use tobj::{load_obj, Material, Model};

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
    let (gl, mut window, mut event_loop, _gl_context) =
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

struct ComputedMatrices {
    mvp: glm::Mat4,
    model: glm::Mat4,
    view: glm::Mat4,
    right: glm::Vec3,
    front: glm::Vec3,
}
fn compute_matrices(
    position: glm::Vec3,
    rotation: glm::Vec2,
    fov: f32,
    aspect_ratio: f32,
) -> ComputedMatrices {
    let z_near = 0.1;
    let z_far = 100.0;
    let projection = glm::ext::perspective(fov, aspect_ratio, z_near, z_far);

    let (cx, sx) = (glm::cos(rotation.x), glm::sin(rotation.x));
    let (cy, sy) = (glm::cos(rotation.y), glm::sin(rotation.y));
    let direction = glm::vec3(cy * sx, sy, cy * cx);
    let right_angle = rotation.x - std::f32::consts::FRAC_PI_2;
    let right = glm::vec3(glm::sin(right_angle), 0.0, glm::cos(right_angle));
    let up = glm::cross(right, direction);
    let front = -glm::cross(right, glm::vec3(0.0, 1.0, 0.0));

    let view = glm::ext::look_at(position, position + direction, up);
    let model = MAT4_ONE;
    let mvp = projection * view * model;
    ComputedMatrices {
        mvp,
        model,
        view,
        right,
        front,
    }
}

struct BakedMeshes {
    vertices: Vec<f32>,
    offsets: Vec<i32>,
    lengths: Vec<i32>,
    uvs: Vec<f32>,
    normals: Vec<f32>,
}
unsafe fn bake_meshes(models: Vec<Model>) -> BakedMeshes {
    let mut models = models
        .iter()
        .filter(|m| {
            if m.mesh.material_id.is_some() {
                true
            } else {
                println!("Skipped model without material_id: {}", m.name);
                false
            }
        })
        .collect::<Vec<_>>();
    models.sort_by_cached_key(|m| m.mesh.material_id);
    let mut vertices = Vec::new();
    let mut offsets = Vec::new();
    let mut lengths = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    // NOTE: models without material_id is ignored for now
    let mut offset = 0;
    let mut length = 0;
    let mut prev_mat_id = models[0].mesh.material_id.unwrap();

    for model in models {
        let m = &model.mesh;
        let mat_id = m.material_id.unwrap();
        if mat_id != prev_mat_id {
            offsets.push(offset as i32);
            lengths.push(length as i32);
            offset += length;
            length = 0;
            prev_mat_id = mat_id;
        }
        vertices.append(
            &mut m
                .indices
                .iter()
                .map(|&i| i as usize)
                .flat_map(|i| {
                    [
                        m.positions[i * 3],
                        m.positions[i * 3 + 1],
                        m.positions[i * 3 + 2],
                    ]
                })
                .collect::<Vec<_>>(),
        );
        uvs.append(
            &mut m
                .texcoord_indices
                .iter()
                .map(|&i| i as usize)
                .flat_map(
                    // NOTE: u, 1-v opengl-tutorial says it's DirectX format, but it also works
                    // with blender cube, so assume all models are in this format
                    |i| [m.texcoords[2 * i], 1.0 - m.texcoords[2 * i + 1]],
                )
                .collect::<Vec<_>>(),
        );
        normals.append(
            &mut m
                .normal_indices
                .iter()
                .map(|&i| i as usize)
                .flat_map(|i| [m.normals[3 * i], m.normals[3 * i + 1], m.normals[3 * i + 2]])
                .collect::<Vec<_>>(),
        );
        length += m.indices.len();
        println!("Loaded model {}", model.name);
    }
    offsets.push(offset as i32);
    lengths.push(length as i32);
    BakedMeshes {
        vertices,
        offsets,
        lengths,
        uvs,
        normals,
    }
}

unsafe fn load_textures(
    gl: &glow::Context,
    materials: &Vec<Material>,
) -> Vec<Option<glow::NativeTexture>> {
    use std::fs::File;
    use std::io::BufReader;
    let mut textures = Vec::new();
    for tx0 in materials {
        if tx0.diffuse_texture.is_none() {
            textures.push(None);
            continue;
        }
        let tx_path = tx0.diffuse_texture.as_ref().unwrap();
        // NOTE consider safety
        let tx = Some(gl.create_texture().unwrap());
        gl.bind_texture(glow::TEXTURE_2D, tx);
        let txr_img = image::load(
            BufReader::new(File::open(tx_path).unwrap()),
            image::ImageFormat::Png,
        )
        .unwrap()
        .into_rgba8();
        let txr_data = txr_img.as_flat_samples().samples;
        let (w, h) = (txr_img.width() as i32, txr_img.height() as i32);
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0, // level of detail TODO: mipmapping
            glow::RGB as i32,
            w,
            h,
            0, // border. literally must be zero. always
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(txr_data),
        );
        // NOTE: releated to mipmapping
        // see glTexParameter#GL_TEXTURE_MIN_FILTER, glTexParameter#GL_TEXTURE_MAG_FILTER
        // (khronos)
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
        textures.push(tx);
    }
    textures
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
