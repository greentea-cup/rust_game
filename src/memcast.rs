pub unsafe fn slice_as_u8<T>(data: &[T]) -> &[u8] {
    std::slice::from_raw_parts(
        data.as_ptr() as *const u8,
        data.len() * core::mem::size_of::<T>(),
    )
}

pub unsafe fn mat4_as_vec(a: glm::Mat4) -> &'static [f32] {
    std::slice::from_raw_parts(a.as_array().as_ptr() as *const f32, 16)
}
