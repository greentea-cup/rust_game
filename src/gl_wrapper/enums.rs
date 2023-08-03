use super::*;

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
pub enum GLShaderType {
    Compute,
    Vertex,
    TessControl,
    TessEvaluation,
    Geometry,
    Fragment,
}
impl From<GLShaderType> for u32 {
    fn from(value: GLShaderType) -> u32 {
        use GLShaderType::*;
        match value {
            Compute => glow::COMPUTE_SHADER,
            Vertex => glow::VERTEX_SHADER,
            TessControl => glow::TESS_CONTROL_SHADER,
            TessEvaluation => glow::TESS_EVALUATION_SHADER,
            Geometry => glow::GEOMETRY_SHADER,
            Fragment => glow::FRAGMENT_SHADER,
        }
    }
}

macro_rules! uniform_impl {
    (
        $type:ty,
        $func:ident,
        $a:ident,
        $($bs:expr),+) => {
        impl GLUniformType for $type {}
        impl GLUniform<'_, $type> {
            pub fn set(&self, $a: $type) {
                unsafe {
                    self.raw_gl().$func(
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
        impl GLUniform<'_, $type> {
            pub fn set(&self, $a: $type, transpose: bool) {
                unsafe {
                    self.raw_gl().$func(
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
