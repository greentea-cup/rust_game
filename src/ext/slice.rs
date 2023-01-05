pub trait AsSlice<T> {
    unsafe fn as_slice<U>(&self) -> &[U];
}

pub trait AsSliceMut<T> {
    unsafe fn as_slice_mut<U>(&mut self) -> &mut [U];
}

impl<T> AsSlice<T> for &[T] {
    unsafe fn as_slice<U>(&self) -> &[U] {
        std::slice::from_raw_parts(
            self.as_ptr() as *const U,
            self.len() * std::mem::size_of::<T>(),
        )
    }
}

impl<T> AsSliceMut<T> for &mut [T] {
    unsafe fn as_slice_mut<U>(&mut self) -> &mut [U] {
        std::slice::from_raw_parts_mut(
            self.as_ptr() as *mut U,
            self.len() * std::mem::size_of::<T>(),
        )
    }
}
