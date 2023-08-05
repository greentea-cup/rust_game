#![allow(dead_code)]
pub use self::enums::*;
use crate::memcast::*;
pub mod enums;
use glm::*;
use glow::HasContext;
use std::marker::PhantomData;

pub struct GLWrapper {
    gl: glow::Context,
    #[allow(unused)]
    gl_context: sdl2::video::GLContext,
}

#[derive(Clone, Copy)]
pub struct GLShader {
    shader: glow::NativeShader,
    shader_type: GLShaderType,
}
impl GLShader {
    pub fn raw(&self) -> glow::NativeShader {
        self.shader
    }
}

pub struct GLProgram<'a> {
    gl: &'a GLWrapper,
    program: glow::Program,
}
impl<'a> GLProgram<'a> {
    fn raw_gl(&self) -> &glow::Context {
        self.gl.raw()
    }

    pub fn raw(&self) -> glow::NativeProgram {
        self.program
    }

    pub fn attach_shader(&self, shader: GLShader) {
        unsafe { self.raw_gl().attach_shader(self.program, shader.shader) }
    }

    pub fn detach_shader(&self, shader: GLShader) {
        unsafe { self.raw_gl().detach_shader(self.program, shader.shader) }
    }

    pub fn link(&self) -> Result<(), String> {
        unsafe {
            self.raw_gl().link_program(self.program);
            if !self.raw_gl().get_program_link_status(self.program) {
                Err(self.raw_gl().get_program_info_log(self.program))
            } else {
                Ok(())
            }
        }
    }

    pub fn get_uniform<T: GLUniformType>(&'a self, name: &'a str) -> GLUniform<'a, T> {
        let location = unsafe { self.raw_gl().get_uniform_location(self.program, name) };
        GLUniform::<T> {
            program: self,
            name,
            location,
            phantom: PhantomData,
        }
    }
}
impl Drop for GLProgram<'_> {
    fn drop(&mut self) {
        unsafe {
            self.raw_gl().delete_program(self.program);
        }
    }
}

#[derive(Clone, Copy)]
pub struct GLVertexAttribute<'a> {
    gl: &'a GLWrapper,
    index: u32,
    target: GLBufferTarget,
    size: i32, // 1, 2, 3, 4
    buffer: glow::Buffer,
    data_type: GLType,
}

impl GLVertexAttribute<'_> {
    fn raw_gl(&self) -> &glow::Context {
        self.gl.raw()
    }

    pub fn enable(&self, normalized: bool, stride: i32, offset: i32) {
        unsafe {
            self.raw_gl().enable_vertex_attrib_array(self.index);
            self.gl.bind_buffer(self.target, self.buffer);
            self.raw_gl().vertex_attrib_pointer_f32(
                self.index,
                self.size,
                self.data_type.into(),
                normalized,
                stride,
                offset,
            );
        }
    }
    pub fn write<T>(&self, data: &[T], usage: GLBufferUsage) {
        self.gl
            .write_to_buffer(self.target, self.buffer, data, usage);
    }
    pub fn disable(&self) {
        unsafe { self.raw_gl().disable_vertex_attrib_array(self.index) }
    }
}

pub trait GLUniformType {}
// GLUniformType impls are in enums module
pub struct GLUniform<'a, T: GLUniformType> {
    program: &'a GLProgram<'a>,
    name: &'a str,
    location: Option<glow::UniformLocation>,
    phantom: PhantomData<T>,
    // NOTE: see GLWrapper::get_uniform
}
impl<T: GLUniformType> GLUniform<'_, T> {
    pub fn name(&self) -> &str {
        self.name
    }
    fn gl(&self) -> &'_ GLWrapper {
        self.program.gl
    }
    fn raw_gl(&self) -> &glow::Context {
        self.gl().raw()
    }
}

// TODO
#[derive(Clone, Copy)]
pub struct GLTexture<'a> {
    gl: &'a GLWrapper,
    texture: glow::Texture,
    target: GLTextureTarget,
}
pub struct GLTextureData<'a> {
    pub level_of_detail: u32,
    pub internal_format: GLColor,
    pub width: u32,
    pub height: u32,
    pub data_format: GLColor,
    pub data_type: GLType,
    pub data: &'a [u8],
}
impl GLTexture<'_> {
    fn raw_gl(&self) -> &glow::Context {
        self.gl.raw()
    }
    pub fn bind(&self) {
        unsafe {
            self.raw_gl()
                .bind_texture(self.target.into(), Some(self.texture));
        }
    }

    pub fn write(&self, data: GLTextureData<'_>) {
        self.bind();
        let GLTextureData {
            level_of_detail,
            internal_format,
            width,
            height,
            data_format,
            data_type,
            data,
        } = data;
        unsafe {
            self.raw_gl().tex_image_2d(
                self.target.into(),
                level_of_detail as i32,
                internal_format.into(),
                width as i32,
                height as i32,
                0, // border
                data_format.into(),
                data_type.into(),
                Some(data),
            )
        }
    }

    pub fn min_filter(&self, value: GLTextureMinFilter) {
        self.parameter_i32(GLTextureParameter::MinFilter, value.into());
    }
    pub fn mag_filter(&self, value: GLTextureMagFilter) {
        self.parameter_i32(GLTextureParameter::MagFilter, value.into());
    }

    pub fn parameter_i32(&self, param: GLTextureParameter, value: i32) {
        self.bind();
        unsafe {
            self.raw_gl()
                .tex_parameter_i32(self.target.into(), param.into(), value)
        }
    }
    pub fn parameter_f32(&self, param: GLTextureParameter, value: f32) {
        self.bind();
        unsafe {
            self.raw_gl()
                .tex_parameter_f32(self.target.into(), param.into(), value);
        }
    }
    pub fn parameter_i32s(&self, param: GLTextureParameter, values: &[i32]) {
        self.bind();
        unsafe {
            self.raw_gl()
                .tex_parameter_i32_slice(self.target.into(), param.into(), values)
        }
    }
    pub fn parameter_f32s(&self, param: GLTextureParameter, values: &[f32]) {
        self.bind();
        unsafe {
            self.raw_gl()
                .tex_parameter_f32_slice(self.target.into(), param.into(), values)
        }
    }
}

#[allow(unused)]
impl GLWrapper {
    pub fn new(gl: glow::Context, gl_context: sdl2::video::GLContext) -> Self {
        GLWrapper { gl, gl_context }
    }
    pub fn raw(&self) -> &glow::Context {
        &self.gl
    }

    pub fn bind_buffer(&self, target: GLBufferTarget, buffer: glow::Buffer) {
        unsafe {
            self.gl.bind_buffer(target.into(), Some(buffer));
        }
    }
    pub fn unbind_buffer(&self, target: GLBufferTarget) {
        unsafe {
            self.gl.bind_buffer(target.into(), None);
        }
    }
    pub fn write_buffer<T>(&self, target: GLBufferTarget, data: &[T], usage: GLBufferUsage) {
        unsafe {
            self.gl
                .buffer_data_u8_slice(target.into(), as_bytes(data), usage.into());
        }
    }
    pub fn write_to_buffer<T>(
        &self,
        target: GLBufferTarget,
        buffer: glow::Buffer,
        data: &[T],
        usage: GLBufferUsage,
    ) {
        self.bind_buffer(target, buffer);
        self.write_buffer(target, data, usage);
    }

    pub fn bind_vertex_array(&self, vertex_array: glow::VertexArray) {
        unsafe {
            self.gl.bind_vertex_array(Some(vertex_array));
        }
    }
    pub fn unbind_vertex_array(&self) {
        unsafe {
            self.gl.bind_vertex_array(None);
        }
    }

    pub fn get_vertex_attribute(
        &self,
        index: u32,
        target: GLBufferTarget,
        size: i32,
        data_type: GLType,
    ) -> Result<GLVertexAttribute, String> {
        let buffer = unsafe { self.gl.create_buffer()? };
        Ok(GLVertexAttribute {
            gl: self,
            index,
            target,
            buffer,
            size,
            data_type,
        })
    }

    pub fn create_program(&self) -> Result<GLProgram<'_>, String> {
        let program = unsafe { self.gl.create_program()? };
        Ok(GLProgram { gl: self, program })
    }

    pub fn set_program(&self, program: &GLProgram<'_>) {
        unsafe { self.gl.use_program(Some(program.program)) }
    }

    pub fn reset_program(&self) {
        unsafe { self.gl.use_program(None) }
    }

    pub fn create_blank_shader(&self, shader_type: GLShaderType) -> Result<GLShader, String> {
        let shader = unsafe { self.gl.create_shader(shader_type.into())? };
        Ok(GLShader {
            shader,
            shader_type,
        })
    }

    pub fn compile_shader(&self, shader: GLShader) -> Result<(), String> {
        let shader = shader.shader;
        unsafe {
            self.gl.compile_shader(shader);
            if !self.gl.get_shader_compile_status(shader) {
                Err(self.gl.get_shader_info_log(shader))
            } else {
                Ok(())
            }
        }
    }

    pub fn create_shader(
        &self,
        shader_type: GLShaderType,
        source: &str,
    ) -> Result<GLShader, String> {
        unsafe {
            let shader = self.create_blank_shader(shader_type)?;
            self.gl.shader_source(shader.shader, source);
            self.compile_shader(shader)?;
            Ok(shader)
        }
    }

    pub fn create_texture(&self, target: GLTextureTarget) -> Result<GLTexture, String> {
        let texture = unsafe { self.gl.create_texture()? };
        Ok(GLTexture {
            gl: self,
            texture,
            target,
        })
    }
}
