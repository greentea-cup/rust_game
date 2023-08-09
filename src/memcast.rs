#![allow(dead_code)]

pub fn as_bytes<T>(data: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * core::mem::size_of::<T>(),
        )
    }
}

pub fn slice_cast<T, U>(data: &[T], len: usize) -> &[U] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const U, len) }
}

pub fn mat2_as_array(data: glm::Mat2) -> [f32; 4] {
    [data[0][0], data[0][1], data[1][0], data[1][1]]
}

pub fn mat3_as_array(data: glm::Mat3) -> [f32; 9] {
    [
        data[0][0], data[0][1], data[0][2], data[1][0], data[1][1], data[1][2], data[2][0],
        data[2][1], data[2][2],
    ]
}

pub fn mat4_as_array(data: glm::Mat4) -> [f32; 16] {
    [
        data[0][0], data[0][1], data[0][2], data[0][3], data[1][0], data[1][1], data[1][2],
        data[1][3], data[2][0], data[2][1], data[2][2], data[2][3], data[3][0], data[3][1],
        data[3][2], data[3][3],
    ]
}
