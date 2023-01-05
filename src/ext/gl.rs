use super::slice::{AsSlice, AsSliceMut};
use glow::{Context, HasContext};
use sdl2::{
    video::{GLContext, Window},
    EventPump,
};

pub trait ContextExt: HasContext {
    unsafe fn compile_shader_status(&self, shader: Self::Shader) -> Result<(), String>;
    unsafe fn link_program_status(&self, program: Self::Program) -> Result<(), String>;

    unsafe fn some_use_program(&self, program: Self::Program);
    unsafe fn none_use_program(&self);
    unsafe fn some_bind_buffer(&self, target: u32, buffer: Self::Buffer);
    unsafe fn none_bind_buffer(&self, target: u32);
    unsafe fn some_bind_vertex_array(&self, vertex_array: Self::VertexArray);
    unsafe fn none_bind_vertex_array(&self);

    unsafe fn clear_color_buffer_bit(&self);

    unsafe fn buffer_data<T>(&self, target: u32, data: &[T], usage: u32);
    unsafe fn buffer_sub_data<T>(&self, target: u32, offset: i32, src_data: &[T]);
    unsafe fn get_buffer_sub_data_t<T>(&self, target: u32, offset: i32, dst_data: &mut [T]);
    unsafe fn some_buffer_storage<T>(&self, target: u32, size: i32, data: &[T], flags: u32);

    // unsafe fn clear_buffer<T>(&self, target: u32, draw_buffer: u32, values: &[T]); // doesn't work

    unsafe fn some_framebuffer_renderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffer_target: u32,
        renderbuffer: Self::Renderbuffer,
    );
    unsafe fn none_framebuffer_renderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffer_target: u32,
    );
}

impl<HasContextT: HasContext> ContextExt for HasContextT {
    unsafe fn compile_shader_status(&self, shader: Self::Shader) -> Result<(), String> {
        self.compile_shader(shader);
        match self.get_shader_compile_status(shader) {
            true => Ok(()),
            false => Err(self.get_shader_info_log(shader)),
        }
    }
    unsafe fn link_program_status(&self, program: Self::Program) -> Result<(), String> {
        self.link_program(program);
        match self.get_program_link_status(program) {
            true => Ok(()),
            false => Err(self.get_program_info_log(program)),
        }
    }

    unsafe fn some_use_program(&self, program: Self::Program) {
        self.use_program(Some(program));
    }
    unsafe fn none_use_program(&self) {
        self.use_program(None);
    }
    unsafe fn some_bind_buffer(&self, target: u32, buffer: Self::Buffer) {
        self.bind_buffer(target, Some(buffer));
    }
    unsafe fn none_bind_buffer(&self, target: u32) {
        self.bind_buffer(target, None);
    }
    unsafe fn some_bind_vertex_array(&self, vertex_array: Self::VertexArray) {
        self.bind_vertex_array(Some(vertex_array));
    }
    unsafe fn none_bind_vertex_array(&self) {
        self.bind_vertex_array(None);
    }

    unsafe fn clear_color_buffer_bit(&self) {
        self.clear(glow::COLOR_BUFFER_BIT);
    }

    unsafe fn buffer_data<T>(&self, target: u32, data: &[T], usage: u32) {
        self.buffer_data_u8_slice(target, data.as_slice(), usage);
    }
    unsafe fn buffer_sub_data<T>(&self, target: u32, offset: i32, src_data: &[T]) {
        self.buffer_sub_data_u8_slice(target, offset, src_data.as_slice());
    }
    unsafe fn get_buffer_sub_data_t<T>(&self, target: u32, offset: i32, mut dst_data: &mut [T]) {
        self.get_buffer_sub_data(target, offset, dst_data.as_slice_mut());
    }
    unsafe fn some_buffer_storage<T>(&self, target: u32, size: i32, data: &[T], flags: u32) {
        self.buffer_storage(target, size, Some(data.as_slice()), flags);
    }

    // unsafe fn clear_buffer<T>(&self, target: u32, draw_buffer: u32, values: &[T]); // no union type contraints found

    unsafe fn some_framebuffer_renderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffer_target: u32,
        renderbuffer: Self::Renderbuffer,
    ) {
        self.framebuffer_renderbuffer(target, attachment, renderbuffer_target, Some(renderbuffer));
    }
    unsafe fn none_framebuffer_renderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffer_target: u32,
    ) {
        self.framebuffer_renderbuffer(target, attachment, renderbuffer_target, None);
    }
    
    // unsafe fn framebuffer_texture(
    //     &self,
    //     target: u32,
    //     attachment: u32,
    //     texture: Option<Self::Texture>,
    //     level: i32,
    // );

    // unsafe fn framebuffer_texture_2d(
    //     &self,
    //     target: u32,
    //     attachment: u32,
    //     texture_target: u32,
    //     texture: Option<Self::Texture>,
    //     level: i32,
    // );

    // unsafe fn framebuffer_texture_3d(
    //     &self,
    //     target: u32,
    //     attachment: u32,
    //     texture_target: u32,
    //     texture: Option<Self::Texture>,
    //     level: i32,
    //     layer: i32,
    // );

    // unsafe fn framebuffer_texture_layer(
    //     &self,
    //     target: u32,
    //     attachment: u32,
    //     texture: Self::Texture,
    //     level: i32, // clarification needed
    //     layer: i32,
    // );
    //
}

// #[cfg()]
/*
trait HasContext_ {
    unsafe fn framebuffer_texture(
        &self,
        target: u32,
        attachment: u32,
        texture: Option<Self::Texture>,
        level: i32,
    );

    unsafe fn framebuffer_texture_2d(
        &self,
        target: u32,
        attachment: u32,
        texture_target: u32,
        texture: Option<Self::Texture>,
        level: i32,
    );

    unsafe fn framebuffer_texture_3d(
        &self,
        target: u32,
        attachment: u32,
        texture_target: u32,
        texture: Option<Self::Texture>,
        level: i32,
        layer: i32,
    );

    unsafe fn framebuffer_texture_layer(
        &self,
        target: u32,
        attachment: u32,
        texture: Option<Self::Texture>,
        level: i32,
        layer: i32,
    );

    unsafe fn front_face(&self, value: u32);

    unsafe fn get_error(&self) -> u32;

    unsafe fn get_tex_parameter_i32(&self, target: u32, parameter: u32) -> i32;

    unsafe fn get_buffer_parameter_i32(&self, target: u32, parameter: u32) -> i32;

    unsafe fn get_parameter_i32(&self, parameter: u32) -> i32;

    unsafe fn get_parameter_i32_slice(&self, parameter: u32, out: &mut [i32]);

    unsafe fn get_parameter_f32(&self, parameter: u32) -> f32;

    unsafe fn get_parameter_f32_slice(&self, parameter: u32, out: &mut [f32]);

    unsafe fn get_parameter_indexed_i32(&self, parameter: u32, index: u32) -> i32;

    unsafe fn get_parameter_indexed_string(&self, parameter: u32, index: u32) -> String;

    unsafe fn get_parameter_string(&self, parameter: u32) -> String;

    unsafe fn get_active_uniform_block_parameter_i32(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
        parameter: u32,
    ) -> i32;

    unsafe fn get_active_uniform_block_parameter_i32_slice(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
        parameter: u32,
        out: &mut [i32],
    );

    unsafe fn get_active_uniform_block_name(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
    ) -> String;

    unsafe fn get_uniform_location(
        &self,
        program: Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation>;

    unsafe fn get_attrib_location(&self, program: Self::Program, name: &str) -> Option<u32>;

    unsafe fn bind_attrib_location(&self, program: Self::Program, index: u32, name: &str);

    unsafe fn get_active_attributes(&self, program: Self::Program) -> u32;

    unsafe fn get_active_attribute(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveAttribute>;

    unsafe fn get_sync_status(&self, fence: Self::Fence) -> u32;

    unsafe fn is_sync(&self, fence: Self::Fence) -> bool;

    unsafe fn renderbuffer_storage(
        &self,
        target: u32,
        internal_format: u32,
        width: i32,
        height: i32,
    );

    unsafe fn renderbuffer_storage_multisample(
        &self,
        target: u32,
        samples: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    );

    unsafe fn sampler_parameter_f32(&self, sampler: Self::Sampler, name: u32, value: f32);

    unsafe fn sampler_parameter_f32_slice(&self, sampler: Self::Sampler, name: u32, value: &[f32]);

    unsafe fn sampler_parameter_i32(&self, sampler: Self::Sampler, name: u32, value: i32);

    unsafe fn generate_mipmap(&self, target: u32);

    unsafe fn tex_image_1d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        border: i32,
        format: u32,
        ty: u32,
        pixels: Option<&[u8]>,
    );

    unsafe fn compressed_tex_image_1d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        border: i32,
        image_size: i32,
        pixels: &[u8],
    );

    unsafe fn tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        ty: u32,
        pixels: Option<&[u8]>,
    );

    unsafe fn tex_image_2d_multisample(
        &self,
        target: u32,
        samples: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        fixed_sample_locations: bool,
    );

    unsafe fn compressed_tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        image_size: i32,
        pixels: &[u8],
    );

    unsafe fn tex_image_3d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        ty: u32,
        pixels: Option<&[u8]>,
    );

    unsafe fn compressed_tex_image_3d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        image_size: i32,
        pixels: &[u8],
    );

    unsafe fn tex_storage_1d(&self, target: u32, levels: i32, internal_format: u32, width: i32);

    unsafe fn tex_storage_2d(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    );

    unsafe fn tex_storage_2d_multisample(
        &self,
        target: u32,
        samples: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        fixed_sample_locations: bool,
    );

    unsafe fn tex_storage_3d(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    );

    unsafe fn get_uniform_i32(
        &self,
        program: Self::Program,
        location: &Self::UniformLocation,
        v: &mut [i32],
    );

    unsafe fn get_uniform_f32(
        &self,
        program: Self::Program,
        location: &Self::UniformLocation,
        v: &mut [f32],
    );

    unsafe fn uniform_1_i32(&self, location: Option<&Self::UniformLocation>, x: i32);

    unsafe fn uniform_2_i32(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32);

    unsafe fn uniform_3_i32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
        z: i32,
    );

    unsafe fn uniform_4_i32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
        z: i32,
        w: i32,
    );

    unsafe fn uniform_1_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]);

    unsafe fn uniform_2_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]);

    unsafe fn uniform_3_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]);

    unsafe fn uniform_4_i32_slice(&self, location: Option<&Self::UniformLocation>, v: &[i32]);

    unsafe fn uniform_1_u32(&self, location: Option<&Self::UniformLocation>, x: u32);

    unsafe fn uniform_2_u32(&self, location: Option<&Self::UniformLocation>, x: u32, y: u32);

    unsafe fn uniform_3_u32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
        z: u32,
    );

    unsafe fn uniform_4_u32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
        z: u32,
        w: u32,
    );

    unsafe fn uniform_1_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]);

    unsafe fn uniform_2_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]);

    unsafe fn uniform_3_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]);

    unsafe fn uniform_4_u32_slice(&self, location: Option<&Self::UniformLocation>, v: &[u32]);

    unsafe fn uniform_1_f32(&self, location: Option<&Self::UniformLocation>, x: f32);

    unsafe fn uniform_2_f32(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32);

    unsafe fn uniform_3_f32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
        z: f32,
    );

    unsafe fn uniform_4_f32(
        &self,
        location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    );

    unsafe fn uniform_1_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]);

    unsafe fn uniform_2_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]);

    unsafe fn uniform_3_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]);

    unsafe fn uniform_4_f32_slice(&self, location: Option<&Self::UniformLocation>, v: &[f32]);

    unsafe fn uniform_matrix_2_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_3_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn uniform_matrix_4_f32_slice(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    );

    unsafe fn unmap_buffer(&self, target: u32);

    unsafe fn cull_face(&self, value: u32);

    unsafe fn color_mask(&self, red: bool, green: bool, blue: bool, alpha: bool);

    unsafe fn color_mask_draw_buffer(
        &self,
        buffer: u32,
        red: bool,
        green: bool,
        blue: bool,
        alpha: bool,
    );

    unsafe fn depth_mask(&self, value: bool);

    unsafe fn blend_color(&self, red: f32, green: f32, blue: f32, alpha: f32);

    unsafe fn line_width(&self, width: f32);

    unsafe fn map_buffer_range(
        &self,
        target: u32,
        offset: i32,
        length: i32,
        access: u32,
    ) -> *mut u8;

    unsafe fn flush_mapped_buffer_range(&self, target: u32, offset: i32, length: i32);

    unsafe fn invalidate_buffer_sub_data(&self, target: u32, offset: i32, length: i32);

    unsafe fn invalidate_framebuffer(&self, target: u32, attachments: &[u32]);

    unsafe fn polygon_offset(&self, factor: f32, units: f32);

    unsafe fn polygon_mode(&self, face: u32, mode: u32);

    unsafe fn finish(&self);

    unsafe fn bind_texture(&self, target: u32, texture: Option<Self::Texture>);

    unsafe fn bind_sampler(&self, unit: u32, sampler: Option<Self::Sampler>);

    unsafe fn active_texture(&self, unit: u32);

    unsafe fn fence_sync(&self, condition: u32, flags: u32) -> Result<Self::Fence, String>;

    unsafe fn tex_parameter_f32(&self, target: u32, parameter: u32, value: f32);

    unsafe fn tex_parameter_i32(&self, target: u32, parameter: u32, value: i32);

    unsafe fn tex_parameter_f32_slice(&self, target: u32, parameter: u32, values: &[f32]);

    unsafe fn tex_parameter_i32_slice(&self, target: u32, parameter: u32, values: &[i32]);

    unsafe fn tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        pixels: PixelUnpackData,
    );

    unsafe fn compressed_tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        pixels: CompressedPixelUnpackData,
    );

    unsafe fn tex_sub_image_3d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        pixels: PixelUnpackData,
    );

    unsafe fn compressed_tex_sub_image_3d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        pixels: CompressedPixelUnpackData,
    );

    unsafe fn depth_func(&self, func: u32);

    unsafe fn depth_range_f32(&self, near: f32, far: f32);

    unsafe fn depth_range_f64(&self, near: f64, far: f64);

    unsafe fn depth_range_f64_slice(&self, first: u32, count: i32, values: &[[f64; 2]]);

    unsafe fn scissor(&self, x: i32, y: i32, width: i32, height: i32);

    unsafe fn scissor_slice(&self, first: u32, count: i32, scissors: &[[i32; 4]]);

    unsafe fn vertex_attrib_divisor(&self, index: u32, divisor: u32);

    unsafe fn vertex_attrib_pointer_f32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        stride: i32,
        offset: i32,
    );

    unsafe fn vertex_attrib_pointer_i32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        stride: i32,
        offset: i32,
    );

    unsafe fn vertex_attrib_pointer_f64(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        stride: i32,
        offset: i32,
    );

    unsafe fn vertex_attrib_format_f32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        relative_offset: u32,
    );

    unsafe fn vertex_attrib_format_i32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        relative_offset: u32,
    );

    unsafe fn vertex_attrib_1_f32(&self, index: u32, x: f32);

    unsafe fn vertex_attrib_2_f32(&self, index: u32, x: f32, y: f32);

    unsafe fn vertex_attrib_3_f32(&self, index: u32, x: f32, y: f32, z: f32);

    unsafe fn vertex_attrib_4_f32(&self, index: u32, x: f32, y: f32, z: f32, w: f32);

    unsafe fn vertex_attrib_1_f32_slice(&self, index: u32, v: &[f32]);

    unsafe fn vertex_attrib_2_f32_slice(&self, index: u32, v: &[f32]);

    unsafe fn vertex_attrib_3_f32_slice(&self, index: u32, v: &[f32]);

    unsafe fn vertex_attrib_4_f32_slice(&self, index: u32, v: &[f32]);

    unsafe fn vertex_attrib_binding(&self, attrib_index: u32, binding_index: u32);

    unsafe fn vertex_binding_divisor(&self, binding_index: u32, divisor: u32);

    unsafe fn viewport(&self, x: i32, y: i32, width: i32, height: i32);

    unsafe fn viewport_f32_slice(&self, first: u32, count: i32, values: &[[f32; 4]]);

    unsafe fn blend_equation(&self, mode: u32);

    unsafe fn blend_equation_draw_buffer(&self, draw_buffer: u32, mode: u32);

    unsafe fn blend_equation_separate(&self, mode_rgb: u32, mode_alpha: u32);

    unsafe fn blend_equation_separate_draw_buffer(
        &self,
        buffer: u32,
        mode_rgb: u32,
        mode_alpha: u32,
    );

    unsafe fn blend_func(&self, src: u32, dst: u32);

    unsafe fn blend_func_draw_buffer(&self, draw_buffer: u32, src: u32, dst: u32);

    unsafe fn blend_func_separate(
        &self,
        src_rgb: u32,
        dst_rgb: u32,
        src_alpha: u32,
        dst_alpha: u32,
    );

    unsafe fn blend_func_separate_draw_buffer(
        &self,
        draw_buffer: u32,
        src_rgb: u32,
        dst_rgb: u32,
        src_alpha: u32,
        dst_alpha: u32,
    );

    unsafe fn stencil_func(&self, func: u32, reference: i32, mask: u32);

    unsafe fn stencil_func_separate(&self, face: u32, func: u32, reference: i32, mask: u32);

    unsafe fn stencil_mask(&self, mask: u32);

    unsafe fn stencil_mask_separate(&self, face: u32, mask: u32);

    unsafe fn stencil_op(&self, stencil_fail: u32, depth_fail: u32, pass: u32);

    unsafe fn stencil_op_separate(&self, face: u32, stencil_fail: u32, depth_fail: u32, pass: u32);

    unsafe fn debug_message_control(
        &self,
        source: u32,
        msg_type: u32,
        severity: u32,
        ids: &[u32],
        enabled: bool,
    );

    unsafe fn debug_message_insert<S>(
        &self,
        source: u32,
        msg_type: u32,
        id: u32,
        severity: u32,
        msg: S,
    ) where
        S: AsRef<str>;

    unsafe fn debug_message_callback<F>(&self, callback: F)
    where
        F: FnMut(u32, u32, u32, u32, &str);

    unsafe fn get_debug_message_log(&self, count: u32) -> Vec<DebugMessageLogEntry>;

    unsafe fn push_debug_group<S>(&self, source: u32, id: u32, message: S)
    where
        S: AsRef<str>;

    unsafe fn pop_debug_group(&self);

    unsafe fn object_label<S>(&self, identifier: u32, name: u32, label: Option<S>)
    where
        S: AsRef<str>;

    unsafe fn get_object_label(&self, identifier: u32, name: u32) -> String;

    unsafe fn object_ptr_label<S>(&self, sync: Self::Fence, label: Option<S>)
    where
        S: AsRef<str>;

    unsafe fn get_object_ptr_label(&self, sync: Self::Fence) -> String;

    unsafe fn get_uniform_block_index(&self, program: Self::Program, name: &str) -> Option<u32>;

    unsafe fn uniform_block_binding(&self, program: Self::Program, index: u32, binding: u32);

    unsafe fn get_shader_storage_block_index(
        &self,
        program: Self::Program,
        name: &str,
    ) -> Option<u32>;

    unsafe fn shader_storage_block_binding(&self, program: Self::Program, index: u32, binding: u32);

    unsafe fn read_buffer(&self, src: u32);

    unsafe fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        gltype: u32,
        pixels: PixelPackData,
    );

    unsafe fn begin_query(&self, target: u32, query: Self::Query);

    unsafe fn end_query(&self, target: u32);

    unsafe fn get_query_parameter_u32(&self, query: Self::Query, parameter: u32) -> u32;

    unsafe fn delete_transform_feedback(&self, transform_feedback: Self::TransformFeedback);

    unsafe fn create_transform_feedback(&self) -> Result<Self::TransformFeedback, String>;

    unsafe fn bind_transform_feedback(
        &self,
        target: u32,
        transform_feedback: Option<Self::TransformFeedback>,
    );

    unsafe fn begin_transform_feedback(&self, primitive_mode: u32);

    unsafe fn end_transform_feedback(&self);

    unsafe fn pause_transform_feedback(&self);

    unsafe fn resume_transform_feedback(&self);

    unsafe fn transform_feedback_varyings(
        &self,
        program: Self::Program,
        varyings: &[&str],
        buffer_mode: u32,
    );

    unsafe fn get_transform_feedback_varying(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveTransformFeedback>;

    unsafe fn memory_barrier(&self, barriers: u32);

    unsafe fn memory_barrier_by_region(&self, barriers: u32);

    unsafe fn bind_image_texture(
        &self,
        unit: u32,
        texture: Self::Texture,
        level: i32,
        layered: bool,
        layer: i32,
        access: u32,
        format: u32,
    );
}
*/

pub fn gl_init(title: &str, width: u32, height: u32) -> (Context, Window, EventPump, GLContext) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core); // also "Compatibility"
    gl_attr.set_context_version(3, 0);
    let window = video
        .window(title, width, height) // options available
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = unsafe {
        glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
    };
    let event_loop = sdl.event_pump().unwrap();
    (gl, window, event_loop, gl_context)
}
