pub fn as_bytes<T>(data: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * core::mem::size_of::<T>(),
        )
    }
}

pub fn vec_as_slice<'a, T: glm::Primitive, U: glm::GenVec<T>>(data: &[U]) -> &'a [T] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const T, data.len() * U::dim()) }
}

pub fn mat2_as_slice<'a>(data: glm::Mat2) -> &'a [f32] {
    unsafe { std::slice::from_raw_parts(data.as_array().as_ptr() as *const f32, 4) }
}

pub fn mat3_as_slice<'a>(data: glm::Mat3) -> &'a [f32] {
    unsafe { std::slice::from_raw_parts(data.as_array().as_ptr() as *const f32, 9) }
}

pub fn mat4_as_slice<'a>(data: glm::Mat4) -> &'a [f32] {
    unsafe { std::slice::from_raw_parts(data.as_array().as_ptr() as *const f32, 16) }
}

pub fn mat2_slice_as_slice<'a>(data: &[glm::Mat2]) -> &'a [f32] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const f32, data.len() * 4) }
}

pub fn mat3_slice_as_slice<'a>(data: &[glm::Mat3]) -> &'a [f32] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const f32, data.len() * 9) }
}

pub fn mat4_slice_as_slice<'a>(data: &[glm::Mat4]) -> &'a [f32] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const f32, data.len() * 16) }
}
