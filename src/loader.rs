use crate::gl_wrapper::*;
use crate::glmc::*;
use tobj::*;

pub struct ComputedMatrices {
    pub view: glm::Mat4,
    pub projection: glm::Mat4,
    pub right: glm::Vec3,
    pub front: glm::Vec3,
}

pub struct BakedMeshes {
    pub indices: Vec<u32>,
    pub vertices: Vec<f32>,
    pub offsets: Vec<i32>,
    pub lengths: Vec<i32>,
    pub uvs: Vec<f32>,
    pub normals: Vec<f32>,
}

pub struct InitializedWindow {
    pub gl: GLWrapper,
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    pub event_loop: sdl2::EventPump,
}

pub fn compute_matrices(
    position: glm::Vec3,
    rotation: glm::Vec2,
    fov: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> ComputedMatrices {
    let projection = glm::ext::perspective(fov, aspect_ratio, z_near, z_far);

    let (cx, sx) = (glm::cos(rotation.x), glm::sin(rotation.x));
    let (cy, sy) = (glm::cos(rotation.y), glm::sin(rotation.y));
    let direction = glm::vec3(cy * sx, sy, cy * cx);
    let right_angle = rotation.x - std::f32::consts::FRAC_PI_2;
    let right = glm::vec3(glm::sin(right_angle), 0.0, glm::cos(right_angle));
    let up = glm::cross(right, direction);
    let front = -glm::cross(right, glm::vec3(0.0, 1.0, 0.0));

    let view = glm::ext::look_at(position, position + direction, up);
    ComputedMatrices {
        view,
        projection,
        right,
        front,
    }
}

pub unsafe fn bake_meshes(models: Vec<Model>, simple_mats: &[usize]) -> BakedMeshes {
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
    models.sort_by_cached_key(|m| {
        (
            // false < true => false is first
            // so materials from simple_mats will be processed first
            !simple_mats.contains(&m.mesh.material_id.unwrap()),
            m.mesh.material_id,
        )
    });
    let mut vertices = Vec::new();
    let mut offsets = Vec::new();
    let mut lengths = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    // NOTE: models without material_id is ignored for now
    let mut offset = 0;
    let mut length = 0;
    let mut prev_mat_id = models[0].mesh.material_id.unwrap();

    let mut idx = 0;
    let mut indices = Vec::new();

    for model in models {
        use std::collections::hash_map::Entry;
        use std::collections::HashMap;
        let m = &model.mesh;
        let mat_id = m.material_id.unwrap();
        if mat_id != prev_mat_id {
            offsets.push(offset as i32);
            lengths.push(length as i32);
            offset += length * std::mem::size_of::<u32>();
            length = 0;
            prev_mat_id = mat_id;
        }

        let data = (0..m.indices.len())
            .map(|i| [m.indices[i], m.texcoord_indices[i], m.normal_indices[i]])
            .collect::<Vec<_>>();
        let mut cache = HashMap::new();
        for x in &data {
            match cache.entry(x) {
                Entry::Occupied(e) => {
                    indices.push(*e.get());
                },
                Entry::Vacant(e) => {
                    e.insert(idx);
                    indices.push(idx);
                    idx += 1;
                    let (p, u, n) = (x[0] as usize, x[1] as usize, x[2] as usize);
                    vertices.extend([
                        m.positions[3 * p],
                        m.positions[3 * p + 1],
                        m.positions[3 * p + 2],
                    ]);
                    uvs.extend([
                        // NOTE: u, 1-v opengl-tutorial says it's DirectX format, but it also works
                        // with blender cube, so assume all models are in this format
                        m.texcoords[2 * u],
                        1.0 - m.texcoords[2 * u + 1],
                    ]);
                    normals.extend([m.normals[3 * n], m.normals[3 * n + 1], m.normals[3 * n + 2]]);
                },
            }
        }

        length += data.len();
        println!("Loaded model {}", model.name);
    }
    offsets.push(offset as i32);
    lengths.push(length as i32);
    BakedMeshes {
        indices,
        vertices,
        offsets,
        lengths,
        uvs,
        normals,
    }
}

pub unsafe fn load_textures<'a>(
    gl: &'a GLWrapper,
    materials: &[Material],
) -> std::collections::HashMap<usize, GLTexture<'a>> {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::BufReader;
    let mut textures = HashMap::new();
    for (i, tx0) in materials.iter().enumerate() {
        if tx0.diffuse_texture.is_none() {
            continue;
        }
        let tx_path = tx0.diffuse_texture.as_ref().unwrap();
        // NOTE consider safety
        let tx = gl
            .create_texture(GLTextureTarget::Texture2D, GLColor::Rgba)
            .unwrap();
        tx.bind();

        let txr_img = image::load(
            BufReader::new(File::open(tx_path).unwrap()),
            image::ImageFormat::Png,
        )
        .unwrap()
        .into_rgba8();
        let txr_data = txr_img.as_flat_samples().samples;
        let (w, h) = (txr_img.width(), txr_img.height());

        tx.write(0, w, h, GLColor::Rgba, GLType::UnsignedByte, txr_data);
        // NOTE: releated to mipmapping
        // see glTexParameter#GL_TEXTURE_MIN_FILTER, glTexParameter#GL_TEXTURE_MAG_FILTER
        // (khronos)
        tx.mag_filter(GLTextureMagFilter::Nearest);
        tx.min_filter(GLTextureMinFilter::Nearest);
        textures.insert(i, tx);
    }
    textures
}

pub unsafe fn load_shaders<'a>(
    gl: &'a GLWrapper,
    shaders: &[(GLShaderType, &std::path::Path)],
) -> GLProgram<'a> {
    use std::fs::read_to_string;
    let program = gl.create_program().unwrap();

    let mut shaders_compiled = Vec::with_capacity(shaders.len());
    for (shader_type, path) in shaders {
        let path_abs = path
            .canonicalize()
            .unwrap_or_else(|_| panic!("Cannot load shader: {}", path.display()));
        let source = read_to_string(&path_abs).unwrap();
        let shader = gl.create_shader(*shader_type, &source).unwrap();
        program.attach_shader(shader);
        shaders_compiled.push(shader);
    }

    program.link().unwrap();

    for shader in shaders_compiled {
        program.detach_shader(shader);
    }
    program
}

pub unsafe fn init_window(width: u32, height: u32) -> Result<InitializedWindow, String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let gl_attr = video.gl_attr();
    // init attrs
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 2);
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
    let gl_wrapper = GLWrapper::new(gl, gl_context);
    let event_loop = sdl.event_pump()?;
    Ok(InitializedWindow {
        gl: gl_wrapper,
        sdl,
        window,
        event_loop,
    })
}
