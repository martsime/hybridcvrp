use std::alloc::{alloc_zeroed, dealloc, Layout};

#[derive(Debug)]
pub struct Matrix<T>
where
    T: Copy,
{
    ptr: *mut T,
    pub rows: usize,
    pub cols: usize,
}

impl<T> Matrix<T>
where
    T: Copy,
{
    pub fn new(rows: usize, cols: usize) -> Self {
        // Allocate memory for the matrix
        let elements = rows * cols;
        let layout = Layout::array::<T>(elements).expect("Failed to create layout for matrix");
        let ptr = unsafe {
            let ptr = alloc_zeroed(layout) as *mut T;
            ptr
        };

        Self { rows, cols, ptr }
    }

    pub fn init(init: T, rows: usize, cols: usize) -> Self {
        let mut matrix = Self::new(rows, cols);
        for row in 0..rows {
            for col in 0..cols {
                matrix.set(row, col, init);
            }
        }
        matrix
    }

    #[inline]
    pub fn get(&self, row: usize, col: usize) -> T {
        unsafe { self.ptr.offset((row * self.cols + col) as isize).read() }
    }

    #[inline]
    pub fn get_mut(&self, row: usize, col: usize) -> &mut T {
        unsafe { &mut *self.ptr.offset((row * self.cols + col) as isize) }
    }

    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        unsafe {
            self.ptr
                .offset((row * self.cols + col) as isize)
                .write(value)
        }
    }

    #[inline]
    pub fn slice(&self, row: usize, col: usize, number: usize) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.ptr.offset((row * self.cols + col) as isize), number)
        }
    }
}

impl<T> Drop for Matrix<T>
where
    T: Copy,
{
    fn drop(&mut self) {
        let layout =
            Layout::array::<T>(self.rows * self.cols).expect("Failed to create layout for matrix");
        unsafe { dealloc(self.ptr as *mut u8, layout) };
    }
}
