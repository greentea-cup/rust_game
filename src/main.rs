mod gl_utils;
mod glmc;
mod loader;
mod memcast;
use crate::glmc::*;
use crate::loader::*;
use crate::gl_utils::*;
use glow::HasContext;
use tobj::load_obj;

fn main() {
    use glm::vec3;
    use std::path::Path;
    let objs_to_load = [
        Path::new("./data/objects/box.obj"),
        Path::new("./data/objects/dice.obj"),
        Path::new("./data/objects/red_crystal.obj"),
        Path::new("./data/objects/green_crystal.obj"),
        Path::new("./data/objects/blue_crystal.obj"),
    ];
    let LoadedModels {
        mut models,
        materials,
    } = prepare_objs(&objs_to_load).unwrap();
    let materials = prepare_materials(&materials);
    let baked = bake_meshes(&mut models, &materials);
    // TODO: split objs and object instances
    // (relation one .obj to many in-game objects
    let objects = [
        (
            0,
            Transform::new(
                vec3(-6., 0.0, 0.),
                vec3(0., 0., 0.),
                vec3(1., 1., 1.),
            ),
        ),
        (
            1,
            Transform::new(
                vec3(-9., 0., 0.),
                vec3(0., 0., 0.),
                vec3(1., 1., 1.),
            ),
        ),
        (
            2,
            Transform::new(
                vec3(0.25, 0., 1.),
                vec3(0., 0., 0.),
                vec3(1., 1., 1.),
            ),
        ),
        (
            3,
            Transform::new(
                vec3(0., -0.25, 0.25),
                vec3(0., 0., 0.),
                vec3(1., 1., 1.),
            ),
        ),
        (
            4,
            Transform::new(
                vec3(0.5, 0.25, 0.5),
                vec3(0., 0., 0.),
                vec3(1., 1., 1.),
            ),
        ),
    ];
    unsafe { main0(baked, &materials, &objects).unwrap() }
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub dissolve: f32,
    pub optical_density: f32,
    pub ambient_texture: Option<Vec<u8>>,
    pub diffuse_texture: Option<Vec<u8>>,
    pub specular_texture: Option<Vec<u8>>,
    pub normal_texture: Option<Vec<u8>>,
    pub shininess_texture: Option<Vec<u8>>,
    pub dissolve_texture: Option<Vec<u8>>,
    pub illumination_model: u8,
    pub is_transparent: bool,
}

fn has_alpha_channel(img: &image::DynamicImage) -> bool {
    use image::DynamicImage;
    matches!(
        img,
        DynamicImage::ImageLumaA8(_)
            | DynamicImage::ImageLumaA16(_)
            | DynamicImage::ImageRgba8(_)
            | DynamicImage::ImageRgba16(_)
            | DynamicImage::ImageRgba32F(_)
    )
}

fn load_texture_data(path: Option<&std::path::Path>) -> Option<Vec<u8>> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path?).ok()?;
    let reader = BufReader::new(file);
    let img = image::load(reader, image::ImageFormat::Png).ok()?;
    println!("{:?}", has_alpha_channel(&img));
    Some(img.to_rgba8().into_flat_samples().samples)
}

fn prepare_materials(materials: &[tobj::Material]) -> Vec<Material> {
    use std::path::Path;
    let mut res = Vec::with_capacity(materials.len());
    for mat in materials.iter() {
        let ambient_texture = load_texture_data(mat.ambient_texture.as_ref().map(Path::new));
        let diffuse_texture = load_texture_data(mat.diffuse_texture.as_ref().map(Path::new));
        let specular_texture = load_texture_data(mat.specular_texture.as_ref().map(Path::new));
        let normal_texture = load_texture_data(mat.normal_texture.as_ref().map(Path::new));
        let dissolve_texture = load_texture_data(mat.dissolve_texture.as_ref().map(Path::new));
        let shininess_texture = load_texture_data(mat.shininess_texture.as_ref().map(Path::new));
        let ambient = mat.ambient.unwrap_or([1., 1., 1.]);
        let diffuse = mat.diffuse.unwrap_or([1., 1., 1.]);
        let specular = mat.specular.unwrap_or([1., 1., 1.]);
        let shininess = mat.shininess.unwrap_or(200.);
        let dissolve = mat.dissolve.unwrap_or(1.);
        let is_transparent = dissolve < 1.;
        let illumination_model = mat.illumination_model.unwrap_or(0);
        let optical_density = mat.optical_density.unwrap_or(1.);
        res.push(Material {
            name: mat.name.as_str().to_owned(),
            ambient,
            diffuse,
            specular,
            shininess,
            dissolve,
            is_transparent,
            ambient_texture,
            diffuse_texture,
            specular_texture,
            dissolve_texture,
            shininess_texture,
            normal_texture,
            illumination_model,
            optical_density,
        });
    }
    res
}

#[derive(Debug)]
struct BakedMeshData {
    vertices: Vec<f32>,
    uvs: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    offsets: Vec<u32>,
    counts: Vec<u32>,
    material_ids: Vec<Option<usize>>,
    opaque: Vec<usize>,
    transparent: Vec<usize>,
}

fn bake_meshes(models: &mut [ModelData], materials: &[Material]) -> BakedMeshData {
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;

    let mut vertices = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut counts = Vec::new();
    let mut offsets = Vec::new();
    let mut material_ids = Vec::new();
    let mut opaque = Vec::new();
    let mut transparent = Vec::new();
    let mut idx = 0;
    let mut offset = 0;
    let mut prev_mid = None;

    let mut cache = HashMap::new();
    // TODO: test glMultiDrawElements to specify spans of same-material objects
    // instead of sorting data
    models.sort_by_cached_key(|model| {
        if model.material_id.is_some() {
            let mid = model.material_id.unwrap();
            let is_transparent = materials[mid].is_transparent;
            (true, is_transparent, mid)
        } else {
            (false, false, 0)
        }
    });

    for (model_index, model) in models.iter().enumerate() {
        println!("{}", model.name);
        {
            let is_transparent =
                model.material_id.is_some() && materials[model.material_id.unwrap()].is_transparent;
            if is_transparent {
                transparent.push(model_index);
            } else {
                opaque.push(model_index);
            }
        }
        if material_ids.is_empty() || (prev_mid != model.material_id) {
            prev_mid = model.material_id;
            material_ids.push(model.material_id);
        }

        const F32S: usize = std::mem::size_of::<f32>();
        let m = &model.mesh;
        let len = m.indices.len();
        // here xs and b_xs are pointing to the same location
        // both f32 and bytes because f32 can't stand as hash key
        // and bytes are not the data needed for result
        let vs = memcast::slice_cast::<f32, [f32; 3]>(&m.vertices, len);
        let us = memcast::slice_cast::<f32, [f32; 2]>(&m.uvs, len);
        let ns = memcast::slice_cast::<f32, [f32; 3]>(&m.normals, len);
        let b_vs = memcast::slice_cast::<f32, [u8; 3 * F32S]>(&m.vertices, len);
        let b_us = memcast::slice_cast::<f32, [u8; 2 * F32S]>(&m.uvs, len);
        let b_ns = memcast::slice_cast::<f32, [u8; 3 * F32S]>(&m.normals, len);

        for i in &m.indices {
            let i = *i as usize;
            let key = (b_vs[i], b_us[i], b_ns[i]);
            let entry = cache.entry(key);
            // find vertex index in cache or else add new and update buffers
            match entry {
                Entry::Occupied(e) => {
                    indices.push(*e.get());
                },
                Entry::Vacant(e) => {
                    let (v, u, n) = (vs[i], us[i], ns[i]);
                    e.insert(idx);
                    indices.push(idx);
                    vertices.extend(v);
                    uvs.extend(u);
                    normals.extend(n);
                    idx += 1;
                },
            }
        }
        let model_length = m.indices.len() as u32;
        counts.push(model_length);
        offsets.push(offset);
        offset += model_length;
    }

    vertices.shrink_to_fit();
    uvs.shrink_to_fit();
    normals.shrink_to_fit();
    indices.shrink_to_fit();
    counts.shrink_to_fit();
    offsets.shrink_to_fit();
    material_ids.shrink_to_fit();
    opaque.shrink_to_fit();
    transparent.shrink_to_fit();

    BakedMeshData {
        vertices,
        uvs,
        normals,
        indices,
        offsets,
        counts,
        material_ids,
        opaque,
        transparent,
    }
}

#[derive(Debug)]
struct MeshData {
    vertices: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u32>,
}

#[derive(Debug)]
struct ModelData {
    mesh: MeshData,
    material_id: Option<usize>,
    name: String,
}

#[derive(Debug)]
struct LoadedModels {
    models: Vec<ModelData>,
    materials: Vec<tobj::Material>,
}

fn prepare_objs(paths: &[&std::path::Path]) -> Result<LoadedModels, String> {
    let load_opts = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
    };
    let mut loaded_models = Vec::with_capacity(paths.len());
    let mut loaded_materials = Vec::with_capacity(paths.len());
    for path in paths {
        let (models, materials) = load_obj(path, &load_opts).map_err(|e| e.to_string())?;
        let mut materials = materials.map_err(|e| e.to_string())?;
        let len = loaded_materials.len();
        for model in models {
            let mesh = MeshData {
                vertices: model.mesh.positions,
                normals: model.mesh.normals,
                uvs: model.mesh.texcoords,
                indices: model.mesh.indices,
            };
            let mid = model.mesh.material_id;
            let res_model = ModelData {
                mesh,
                material_id: mid.map(|i| i + len),
                name: model.name,
            };
            loaded_models.push(res_model);
        }
        loaded_materials.append(&mut materials);
    }
    Ok(LoadedModels {
        models: loaded_models,
        materials: loaded_materials,
    })
}

unsafe fn main0(
    models: BakedMeshData,
    materials: &[Material],
    objects: &[(usize, Transform)],
) -> Result<(), String> {
    let start = std::time::SystemTime::now();
    let (mut width, mut height): (u32, u32) = (800, 600);
    let InitializedWindow {
        gl,
        sdl,
        window,
        mut event_loop,
        gl_context: _gl_context, /* NOTE: should not drop */
    } = init_window(width, height)?;

    let mouse = sdl.mouse();
    mouse.set_relative_mouse_mode(true);

    gl.enable(glow::DEBUG_OUTPUT);
    gl.debug_message_callback(debug_message_callback);

    let mut aspect_ratio = width as f32 / height as f32;
    let fov = glm::radians(45.);
    

    let (shaders, solid_u, transparent_u) = init_shaders(&gl).unwrap();

    let screen_quad_data: &[f32] = &[
        // x, y, z, u, v
        -1.0, -1.0, 0.0, 0.0, 0.0, //
        1.0, -1.0, 0.0, 1.0, 0.0, //
        1.0, 1.0, 0.0, 1.0, 1.0, //
        1.0, 1.0, 0.0, 1.0, 1.0, //
        -1.0, 1.0, 0.0, 0.0, 1.0, //
        -1.0, -1.0, 0.0, 0.0, 0.0, //
    ];

    const F32S: i32 = std::mem::size_of::<f32>() as i32;
    let main_vao = gl.create_vertex_array()?;
    let main_vertices = gl.create_buffer()?;
    let main_uvs = gl.create_buffer()?;
    let main_normals = gl.create_buffer()?;
    let main_element_buf = gl.create_buffer()?;
    {
        // TODO
        gl.bind_vertex_array(Some(main_vao));

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(main_element_buf));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            memcast::as_bytes(&models.indices),
            glow::STATIC_DRAW,
        );

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(main_vertices));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            memcast::as_bytes(&models.vertices),
            glow::STATIC_DRAW,
        );
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(main_uvs));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            memcast::as_bytes(&models.uvs),
            glow::STATIC_DRAW,
        );
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(main_normals));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            memcast::as_bytes(&models.normals),
            glow::STATIC_DRAW,
        );
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, 0, 0);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);
    }
    let screen_vao = gl.create_vertex_array()?;
    let screen_vbo = gl.create_buffer()?;
    {
        gl.bind_vertex_array(Some(screen_vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(screen_vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            memcast::as_bytes(screen_quad_data),
            glow::STATIC_DRAW,
        );
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 5 * F32S, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 5 * F32S, 3 * F32S);
        gl.bind_vertex_array(None);
    }

    // set up framebuffers and their texture attachments
    let opaque_fbo = gl.create_framebuffer()?;
    let transparent_fbo = gl.create_framebuffer()?;
    // attachments opaque
    let opaque_tx = gl.create_texture()?;
    let opaque_params = TextureParams {
        internal_format: glow::RGBA16F,
        format: glow::RGBA,
        data_type: glow::HALF_FLOAT,
        min_filter: Some(glow::LINEAR),
        mag_filter: Some(glow::LINEAR),
    };
    reset_texture(&gl, opaque_tx, &opaque_params, width, height);
    let depth_tx = gl.create_texture()?;
    let depth_params = TextureParams {
        internal_format: glow::DEPTH_COMPONENT,
        format: glow::DEPTH_COMPONENT,
        data_type: glow::FLOAT,
        min_filter: None,
        mag_filter: None,
    };
    reset_texture(&gl, depth_tx, &depth_params, width, height);
    let opaque_fbo_textures = &[
        (glow::COLOR_ATTACHMENT0, opaque_tx),
        (glow::DEPTH_ATTACHMENT, depth_tx),
    ];
    let opaque_fbo_drawbufs = None;
    rebind_framebuffer(&gl, opaque_fbo, opaque_fbo_textures, opaque_fbo_drawbufs)?;
    // attachments transparent
    let accum_tx = gl.create_texture()?;
    let accum_params = TextureParams {
        internal_format: glow::RGBA16F,
        format: glow::RGBA,
        data_type: glow::HALF_FLOAT,
        min_filter: Some(glow::LINEAR),
        mag_filter: Some(glow::LINEAR),
    };
    reset_texture(&gl, accum_tx, &accum_params, width, height);
    let reveal_tx = gl.create_texture()?;
    let reveral_params = TextureParams {
        internal_format: glow::R8,
        format: glow::RED,
        data_type: glow::FLOAT,
        min_filter: Some(glow::LINEAR),
        mag_filter: Some(glow::LINEAR),
    };
    reset_texture(&gl, reveal_tx, &reveral_params, width, height);
    let transparent_fbo_textures: &[(GLTextureAttachment, glow::Texture)] = &[
        (glow::COLOR_ATTACHMENT0, accum_tx),
        (glow::COLOR_ATTACHMENT1, reveal_tx),
        (glow::DEPTH_ATTACHMENT, depth_tx),
    ];
    let transparent_fbo_drawbufs =
        Some::<&[GLDrawBuffer]>(&[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1]);
    rebind_framebuffer(
        &gl,
        transparent_fbo,
        transparent_fbo_textures,
        transparent_fbo_drawbufs,
    )?;
    // transform matrices
    let obj_mtxs = objects
        .iter()
        .map(|(i, mmi)| (*i, model_mat_from(*mmi)))
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
        fullscreen: false,
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
    let _light_position = glm::vec3(4., 3., 3.);

    'render: loop {
        let (w, h) = state.window.size();
        if (w != width) || (h != height) {
            width = w;
            height = h;
            aspect_ratio = width as f32 / height as f32;
            gl.viewport(0, 0, width as i32, height as i32);
            let textures = [
                (opaque_tx, &opaque_params),
                (depth_tx, &depth_params),
                (accum_tx, &accum_params),
                (reveal_tx, &reveral_params),
            ];
            for (tx, params) in textures.iter() {
                reset_texture(&gl, *tx, params, width, height);
            }
            rebind_framebuffer(&gl, opaque_fbo, opaque_fbo_textures, opaque_fbo_drawbufs)?;
            rebind_framebuffer(
                &gl,
                transparent_fbo,
                transparent_fbo_textures,
                transparent_fbo_drawbufs,
            )?;
        }
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

        const DEFAULT_AMBIENT: [f32; 3] = [1., 1., 1.];
        const DEFAULT_DIFFUSE: [f32; 3] = [1., 1., 1.];
        const DEFAULT_SPECULAR: [f32; 3] = [0.5, 0.5, 0.5];
        const DEFAULT_DISSOLVE: f32 = 0.5;

        // render
        // solid
        {
            if state.culling {
                gl.enable(glow::CULL_FACE);
            }
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.depth_mask(true);
            gl.disable(glow::BLEND);
            gl.clear_color(0.1, 0.2, 0.3, 0.);
            // bind opaque buffer
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(opaque_fbo));
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(shaders.solid));

            let mut prev_mid = None;
            for i in &models.opaque {
                let i = *i;
                gl.uniform_matrix_4_f32_slice(
                    solid_u.mvp.as_ref(),
                    false,
                    &memcast::mat4_as_array(vp_mat * obj_mtxs[i].1),
                );
                let mid = models.material_ids[i];
                if (mid != prev_mid) || (i == 0) {
                    prev_mid = mid;
                    if let Some(mat_id) = mid {
                        let mat = &materials[mat_id];
                        gl.uniform_3_f32_slice(solid_u.ambient_color.as_ref(), &mat.ambient);
                        gl.uniform_3_f32_slice(solid_u.diffuse_color.as_ref(), &mat.diffuse);
                        gl.uniform_3_f32_slice(solid_u.specular_color.as_ref(), &mat.specular);
                    } else {
                        // set default materisl
                        gl.uniform_3_f32_slice(solid_u.ambient_color.as_ref(), &DEFAULT_AMBIENT);
                        gl.uniform_3_f32_slice(solid_u.diffuse_color.as_ref(), &DEFAULT_DIFFUSE);
                        gl.uniform_3_f32_slice(solid_u.specular_color.as_ref(), &DEFAULT_SPECULAR);
                    }
                }
                gl.active_texture(glow::TEXTURE1);
                gl.uniform_1_i32(solid_u.diffuse_texture.as_ref(), 1);
                gl.uniform_3_i32(solid_u.opts.as_ref(), 0, 1, 0);
                gl.bind_vertex_array(Some(main_vao));
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(main_element_buf));
                gl.draw_elements(
                    glow::TRIANGLES,
                    models.counts[i] as i32,
                    glow::UNSIGNED_INT,
                    models.offsets[i] as i32,
                );
                draw_calls += 1;
            }
        }
        // transparent
        {
            gl.disable(glow::CULL_FACE);
            gl.depth_mask(false);
            gl.enable(glow::BLEND);
            gl.blend_func_draw_buffer(0, glow::ONE, glow::ONE);
            gl.blend_func_draw_buffer(1, glow::ZERO, glow::ONE_MINUS_SRC_COLOR);
            gl.blend_equation(glow::FUNC_ADD);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(transparent_fbo));
            gl.clear_buffer_f32_slice(glow::COLOR, 0, &[0., 0., 0., 0.]);
            gl.clear_buffer_f32_slice(glow::COLOR, 1, &[1., 1., 1., 1.]);

            gl.use_program(Some(shaders.transparent));

            let mut prev_mid = None;
            for i in &models.transparent {
                let i = *i;
                gl.uniform_matrix_4_f32_slice(
                    transparent_u.mvp.as_ref(),
                    false,
                    &memcast::mat4_as_array(vp_mat * obj_mtxs[i].1),
                );
                let mid = models.material_ids[i];
                if (mid != prev_mid) || (i == 0) {
                    prev_mid = mid;
                    if let Some(mat_id) = mid {
                        let mat = &materials[mat_id];
                        gl.uniform_3_f32_slice(transparent_u.ambient_color.as_ref(), &mat.ambient);
                        gl.uniform_3_f32_slice(transparent_u.diffuse_color.as_ref(), &mat.diffuse);
                        gl.uniform_3_f32_slice(
                            transparent_u.specular_color.as_ref(),
                            &mat.specular,
                        );
                        gl.uniform_1_f32(transparent_u.dissolve.as_ref(), mat.dissolve);
                    } else {
                        // set default materisl
                        gl.uniform_3_f32_slice(
                            transparent_u.ambient_color.as_ref(),
                            &DEFAULT_AMBIENT,
                        );
                        gl.uniform_3_f32_slice(
                            transparent_u.diffuse_color.as_ref(),
                            &DEFAULT_DIFFUSE,
                        );
                        gl.uniform_3_f32_slice(
                            transparent_u.specular_color.as_ref(),
                            &DEFAULT_SPECULAR,
                        );
                        gl.uniform_1_f32(transparent_u.dissolve.as_ref(), DEFAULT_DISSOLVE);
                    }
                }
                gl.active_texture(glow::TEXTURE1);
                gl.uniform_1_i32(transparent_u.diffuse_texture.as_ref(), 1);
                gl.uniform_3_i32(transparent_u.opts.as_ref(), 0, 1, 0);
                gl.bind_vertex_array(Some(main_vao));
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(main_element_buf));
                gl.draw_elements(
                    glow::TRIANGLES,
                    models.counts[i] as i32,
                    glow::UNSIGNED_INT,
                    models.offsets[i] as i32,
                );
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
            gl.bind_vertex_array(Some(screen_vao));
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
            gl.bind_vertex_array(Some(screen_vao));
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

struct GameState<'a> {
    #[allow(unused)]
    gl: &'a glow::Context,
    window: sdl2::video::Window,
    mouse: sdl2::mouse::MouseUtil,
    position: glm::Vec3,
    rotation: glm::Vec2,
    light_power: f32,
    light_intensity: glm::IVec3,
    captured: bool,
    fullscreen: bool,
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
            scancode: Some(Scancode::F11),
            ..
        } => {
            state.fullscreen = !state.fullscreen;
            use sdl2::video::FullscreenType;
            if state.fullscreen {
                if let Err(e) = state.window.set_fullscreen(FullscreenType::Desktop) {
                    eprintln!("Cannot set fullscreen: {}", e);
                }
            } else if let Err(e) = state.window.set_fullscreen(FullscreenType::Off) {
                eprintln!("Cannot exit fullscreen: {}", e);
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
            println!("CUlling: {}", state.culling);
            println!("Draw calls: {}", state.draw_calls);
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
                Scancode::G => state.culling = !state.culling,
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
        Event::MouseWheel { y, .. } if state.captured => {
            state.light_power += 5.0 * y as f32;
            state.light_power = state.light_power.clamp(0.0, 100.0);
        },
        _ => {},
    }
}
