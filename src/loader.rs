use crate::glmc::*;
use glow::HasContext;
use tobj::*;

pub struct ComputedMatrices {
    pub mvp: glm::Mat4,
    pub model: glm::Mat4,
    pub view: glm::Mat4,
    pub right: glm::Vec3,
    pub front: glm::Vec3,
}

pub struct BakedMeshes {
    pub vertices: Vec<f32>,
    pub offsets: Vec<i32>,
    pub lengths: Vec<i32>,
    pub uvs: Vec<f32>,
    pub normals: Vec<f32>,
}

pub struct InitializedWindow {
    pub gl: glow::Context,
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    pub event_loop: sdl2::EventPump,
    pub gl_context: sdl2::video::GLContext,
}

pub type GlowShaderType = u32;

pub fn compute_matrices(
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

pub unsafe fn bake_meshes(models: Vec<Model>) -> BakedMeshes {
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

pub unsafe fn load_textures(
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

pub unsafe fn load_shaders(
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

pub unsafe fn init_window(width: u32, height: u32) -> Result<InitializedWindow, String> {
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
    Ok(InitializedWindow {
        gl,
        sdl,
        window,
        event_loop,
        gl_context,
    })
}
