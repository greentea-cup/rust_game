#![allow(dead_code)]
use crate::memcast::*;
use glm::*;
use glow::HasContext;
use std::marker::PhantomData;

pub struct GLWrapper {
    gl: glow::Context,
    #[allow(unused)]
    gl_context: sdl2::video::GLContext,
}

#[derive(Clone, Copy)]
pub enum GLBufferTarget {
    Array,
    AtomicCounter,
    CopyRead,
    CopyWrite,
    DispatchIndirect,
    DrawIndirect,
    ElementArray,
    PixelPack,
    PixelUnpack,
    Query,
    ShederStorage,
    Texture,
    TransformFeedback,
    Uniform,
}
impl From<GLBufferTarget> for u32 {
    fn from(value: GLBufferTarget) -> u32 {
        use GLBufferTarget::*;
        match value {
            Array => glow::ARRAY_BUFFER,
            AtomicCounter => glow::ATOMIC_COUNTER_BUFFER,
            CopyRead => glow::COPY_READ_BUFFER,
            CopyWrite => glow::COPY_WRITE_BUFFER,
            DispatchIndirect => glow::DISPATCH_INDIRECT_BUFFER,
            DrawIndirect => glow::DRAW_INDIRECT_BUFFER,
            ElementArray => glow::ELEMENT_ARRAY_BUFFER,
            PixelPack => glow::PIXEL_PACK_BUFFER,
            PixelUnpack => glow::PIXEL_UNPACK_BUFFER,
            Query => glow::QUERY_BUFFER,
            ShederStorage => glow::SHADER_STORAGE_BUFFER,
            Texture => glow::TEXTURE_BUFFER,
            TransformFeedback => glow::TRANSFORM_FEEDBACK_BUFFER,
            Uniform => glow::UNIFORM_BUFFER,
        }
    }
}

#[derive(Clone, Copy)]
pub enum GLBufferUsage {
    StreamDraw,
    StreamRead,
    StreamCopy,
    StaticDraw,
    StaticRead,
    StaticCopy,
    DynamicDraw,
    DynamicRead,
    DynamicCopy,
}
impl From<GLBufferUsage> for u32 {
    fn from(value: GLBufferUsage) -> u32 {
        use GLBufferUsage::*;
        match value {
            StreamDraw => glow::STREAM_DRAW,
            StreamRead => glow::STREAM_READ,
            StreamCopy => glow::STREAM_COPY,
            StaticDraw => glow::STATIC_DRAW,
            StaticRead => glow::STATIC_READ,
            StaticCopy => glow::STATIC_COPY,
            DynamicDraw => glow::DYNAMIC_DRAW,
            DynamicRead => glow::DYNAMIC_READ,
            DynamicCopy => glow::DYNAMIC_COPY,
        }
    }
}

#[derive(Clone, Copy)]
pub enum GLType {
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    HalfFloat,
    Float,
    Double,
    Fixed,
    // NOTE: not all types listed
    // for other types use GLWrapper::raw() calls
}

impl From<GLType> for u32 {
    fn from(value: GLType) -> u32 {
        use GLType::*;
        match value {
            Byte => glow::BYTE,
            UnsignedByte => glow::UNSIGNED_BYTE,
            Short => glow::SHORT,
            UnsignedShort => glow::UNSIGNED_SHORT,
            Int => glow::INT,
            UnsignedInt => glow::UNSIGNED_INT,
            HalfFloat => glow::FLOAT,
            Float => glow::FLOAT,
            Double => glow::DOUBLE,
            Fixed => glow::FIXED,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GLVertexAttribute {
    index: u32,
    target: GLBufferTarget,
    size: i32, // 1, 2, 3, 4
    buffer: glow::Buffer,
    data_type: GLType,
}

impl GLVertexAttribute {
    pub fn enable(&self, gl: &GLWrapper, normalized: bool, stride: i32, offset: i32) {
        unsafe {
            gl.raw().enable_vertex_attrib_array(self.index);
            gl.bind_buffer(self.target, self.buffer);
            gl.raw().vertex_attrib_pointer_f32(
                self.index,
                self.size,
                self.data_type.into(),
                normalized,
                stride,
                offset,
            );
        }
    }
    pub fn write<T>(&self, gl: &GLWrapper, data: &[T], usage: GLBufferUsage) {
        gl.write_to_buffer(self.target, self.buffer, data, usage);
    }
    pub fn disable(&self, gl: &GLWrapper) {
        unsafe { gl.raw().disable_vertex_attrib_array(self.index) }
    }
}

pub struct GLUniform<T: GLUniformType> {
    name: String,
    location: Option<glow::UniformLocation>,
    phantom: PhantomData<T>,
    // NOTE: see GLWrapper::get_uniform
}
pub trait GLUniformType {}
macro_rules! uniform_impl {
    (
        $type:ty,
        $func:ident,
        $a:ident,
        $($bs:expr),+) => {
        impl GLUniformType for $type {}
        impl GLUniform<$type> {
            pub fn set(&self, gl: &GLWrapper, $a: $type) {
                unsafe {
                    gl.raw().$func(
                        self.location.as_ref(),
                        $($bs),+
                    );
                }
            }
        }
    };
    (
        $type:ty,
        $func:ident,
        $a:ident,
        transpose;
        $($bs:expr),+) => {
        impl GLUniformType for $type {}
        impl GLUniform<$type> {
            pub fn set(&self, gl: &GLWrapper, $a: $type, transpose: bool) {
                unsafe {
                    gl.raw().$func(
                        self.location.as_ref(),
                        transpose,
                        $($bs),+
                    );
                }
            }
        }
    };
}

// TODO: stop wasting time
uniform_impl!(f32, uniform_1_f32, v, v);
uniform_impl!(Vec2, uniform_2_f32, v, v.x, v.y);
uniform_impl!(Vec3, uniform_3_f32, v, v.x, v.y, v.z);
uniform_impl!(Vec4, uniform_4_f32, v, v.x, v.y, v.z, v.w);
uniform_impl!(&[f32], uniform_1_f32_slice, v, v);
uniform_impl!(&[Vec2], uniform_2_f32_slice, v, vec_as_slice(v));
uniform_impl!(&[Vec3], uniform_3_f32_slice, v, vec_as_slice(v));
uniform_impl!(&[Vec4], uniform_4_f32_slice, v, vec_as_slice(v));

uniform_impl!(i32, uniform_1_i32, v, v);
uniform_impl!(IVec2, uniform_2_i32, v, v.x, v.y);
uniform_impl!(IVec3, uniform_3_i32, v, v.x, v.y, v.z);
uniform_impl!(IVec4, uniform_4_i32, v, v.x, v.y, v.z, v.w);
uniform_impl!(&[i32], uniform_1_i32_slice, v, v);
uniform_impl!(&[IVec2], uniform_2_i32_slice, v, vec_as_slice(v));
uniform_impl!(&[IVec3], uniform_3_i32_slice, v, vec_as_slice(v));
uniform_impl!(&[IVec4], uniform_4_i32_slice, v, vec_as_slice(v));

uniform_impl!(u32, uniform_1_u32, v, v);
uniform_impl!(UVec2, uniform_2_u32, v, v.x, v.y);
uniform_impl!(UVec3, uniform_3_u32, v, v.x, v.y, v.z);
uniform_impl!(UVec4, uniform_4_u32, v, v.x, v.y, v.z, v.w);
uniform_impl!(&[u32], uniform_1_u32_slice, v, v);
uniform_impl!(&[UVec2], uniform_2_u32_slice, v, vec_as_slice(v));
uniform_impl!(&[UVec3], uniform_3_u32_slice, v, vec_as_slice(v));
uniform_impl!(&[UVec4], uniform_4_u32_slice, v, vec_as_slice(v));

uniform_impl!(Mat2, uniform_matrix_2_f32_slice, v, transpose; mat2_as_slice(v));
uniform_impl!(Mat3, uniform_matrix_3_f32_slice, v, transpose; mat3_as_slice(v));
uniform_impl!(Mat4, uniform_matrix_4_f32_slice, v, transpose; mat4_as_slice(v));
uniform_impl!(&[Mat2], uniform_matrix_2_f32_slice, v, transpose; mat2_slice_as_slice(v));
uniform_impl!(&[Mat3], uniform_matrix_3_f32_slice, v, transpose; mat3_slice_as_slice(v));
uniform_impl!(&[Mat4], uniform_matrix_4_f32_slice, v, transpose; mat4_slice_as_slice(v));

pub enum GLTextureTarget {
    Texture1D,
    Texture2D,
    Texture3D,
}
impl From<GLTextureTarget> for u32 {
    fn from(value: GLTextureTarget) -> u32 {
        use GLTextureTarget::*;
        match value {
            Texture1D => glow::TEXTURE_1D,
            Texture2D => glow::TEXTURE_2D,
            Texture3D => glow::TEXTURE_3D,
        }
    }
}
pub struct GLTexture {
    texture: glow::Texture,
    target: GLTextureTarget,
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
            index,
            target,
            buffer,
            size,
            data_type,
        })
    }

    pub fn get_uniform<T: GLUniformType>(
        &self,
        program: glow::Program,
        name: &str,
    ) -> GLUniform<T> {
        let location = unsafe { self.gl.get_uniform_location(program, name) };
        GLUniform::<T> {
            name: name.to_owned(),
            location,
            phantom: PhantomData,
        }
    }

    pub fn get_texture(&self, target: GLTextureTarget) -> Result<GLTexture, String> {
        let texture = unsafe { self.gl.create_texture()? };
        Ok(GLTexture { texture, target })
    }
}
