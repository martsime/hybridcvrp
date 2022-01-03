use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    fmt::Display,
};

use lazysort::SortedBy;

use crate::{
    config::Config,
    models::{Coordinate, Problem},
    utils::FloatCompare,
};

#[derive(Debug)]
pub struct Matrix<T>
where
    T: Copy,
{
    ptr: *mut T,
    pub rows: usize,
    pub cols: usize,
}

impl<T: Copy> Matrix<T> {
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

    #[inline]
    pub fn slice_mut(&self, row: usize, col: usize, number: usize) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.ptr.offset((row * self.cols + col) as isize),
                number,
            )
        }
    }

    pub fn from_mapping(&self, mapping: &[usize]) -> Self {
        let new_size = mapping.len();
        let new_matrix = Self::new(new_size, new_size);
        for i in 0..new_size {
            for j in 0..new_size {
                *new_matrix.get_mut(i, j) = self.get(mapping[i], mapping[j]);
            }
        }
        new_matrix
    }

    pub fn get_max(&self) -> T
    where
        T: PartialOrd,
    {
        *self
            .slice(0, 0, self.rows * self.cols)
            .into_iter()
            .max_by(|&a, &b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}

impl<T: Copy> Clone for Matrix<T> {
    fn clone(&self) -> Self {
        let mut new_matrix = Self::new(self.rows, self.cols);

        for i in 0..self.rows {
            for j in 0..self.cols {
                new_matrix.set(i, j, self.get(i, j));
            }
        }
        new_matrix
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

impl Display for Matrix<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.rows {
            write!(f, "[ ")?;
            for j in 0..self.cols {
                if j < self.cols - 1 {
                    write!(f, "{}, ", self.get(i, j))?;
                } else {
                    writeln!(f, "{} ]", self.get(i, j))?;
                }
            }
        }

        Ok(())
    }
}

/// Calculates the euclidian distance between two coordinates
#[inline]
fn euclidian(c1: &Coordinate, c2: &Coordinate) -> f64 {
    ((c2.lng - c1.lng).powi(2) + (c2.lat - c1.lat).powi(2)).sqrt()
}

/// Builder for the DistanceMatrix
pub struct DistanceMatrixBuilder {
    locations: Vec<Coordinate>,
    precompute: bool,
    rounded: bool,
    input: Option<Vec<Vec<f64>>>,
    max_distance: Option<f64>,
}

impl DistanceMatrixBuilder {
    pub fn new() -> Self {
        Self {
            locations: Vec::new(),
            precompute: false,
            rounded: false,
            input: None,
            max_distance: None,
        }
    }

    pub fn locations(mut self, locations: Vec<Coordinate>) -> Self {
        self.locations = locations;
        self
    }

    pub fn precompute(mut self, precompute: bool) -> Self {
        self.precompute = precompute;
        self
    }

    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    pub fn input(mut self, input: Vec<Vec<f64>>) -> Self {
        self.input = Some(input);
        self
    }

    pub fn build(mut self) -> DistanceMatrix {
        let matrix = if let Some(input) = self.input {
            self.precompute = true;
            let n = self.locations.len();
            let mut matrix = Matrix::new(n, n);
            for (i, row) in input.iter().enumerate() {
                for (j, &distance) in row.iter().enumerate() {
                    let mut distance = distance;
                    if self.rounded {
                        distance = distance.round();
                    }
                    matrix.set(i + 1, j, distance);
                    matrix.set(j, i + 1, distance);
                    match self.max_distance.as_mut() {
                        Some(max_distance) => {
                            if distance.approx_gt(&*max_distance) {
                                *max_distance = distance;
                            }
                        }
                        None => {
                            self.max_distance = Some(distance);
                        }
                    }
                }
            }

            matrix
        } else {
            match self.precompute {
                true => {
                    let n = self.locations.len();
                    let mut matrix = Matrix::new(n, n);

                    // Assumes a symmetic matrix
                    for i in 0..n {
                        for j in (i + 1)..n {
                            let mut distance = euclidian(&self.locations[i], &self.locations[j]);
                            if self.rounded {
                                distance = distance.round();
                            }

                            matrix.set(i, j, distance);
                            matrix.set(j, i, distance);

                            match self.max_distance.as_mut() {
                                Some(max_distance) => {
                                    if distance.approx_gt(&*max_distance) {
                                        *max_distance = distance;
                                    }
                                }
                                None => {
                                    self.max_distance = Some(distance);
                                }
                            }
                        }
                    }
                    matrix
                }
                false => Matrix::new(0, 0),
            }
        };

        DistanceMatrix::new(
            self.locations,
            matrix,
            self.precompute,
            self.rounded,
            self.max_distance,
        )
    }
}

/// Distance matrix.
///
/// Supports lazy evaluation where the distance is calculated every time
/// it is queried, in contrast to precomputing the matrix.
#[derive(Debug)]
pub struct DistanceMatrix {
    locations: Vec<Coordinate>,
    storage: Matrix<f64>,
    precomputed: bool,
    rounded: bool,
    max_distance: Option<f64>,
}

impl DistanceMatrix {
    pub fn new(
        locations: Vec<Coordinate>,
        storage: Matrix<f64>,
        precomputed: bool,
        rounded: bool,
        max_distance: Option<f64>,
    ) -> Self {
        Self {
            locations,
            storage,
            precomputed,
            rounded,
            max_distance,
        }
    }

    pub fn from_input(storage: Matrix<f64>, rounded: bool, max_distance: Option<f64>) -> Self {
        Self {
            locations: Vec::new(),
            storage,
            precomputed: true,
            rounded,
            max_distance,
        }
    }

    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        match self.precomputed {
            true => self.storage.get(row, col),
            false => {
                let mut distance = euclidian(&self.locations[row], &self.locations[col]);
                if self.rounded {
                    distance = distance.round();
                }
                distance
            }
        }
    }

    pub fn get_vec(&self, row: usize, col: usize, number: usize) -> Vec<f64> {
        match self.precomputed {
            true => self
                .storage
                .slice(row, col, number)
                .iter()
                .copied()
                .collect(),
            false => {
                let size = self.size();
                let mut row_index = row;
                let mut col_index = col;
                (0..number)
                    .map(|_| {
                        let value = self.get(row_index, col_index);
                        if col_index < size - 1 {
                            col_index += 1;
                        } else {
                            row_index += 1;
                            col_index = 0;
                        }
                        value
                    })
                    .collect()
            }
        }
    }

    pub fn size(&self) -> usize {
        self.locations.len()
    }

    pub fn max(&self) -> Option<f64> {
        self.max_distance
    }

    pub fn from_mapping(&self, mapping: &[usize]) -> Self {
        let locations = mapping
            .iter()
            .map(|&index| self.locations[index].clone())
            .collect();
        let size = mapping.len();
        let storage = Matrix::new(size, size);
        for i in 0..size {
            for j in 0..size {
                *storage.get_mut(i, j) = self.get(mapping[i], mapping[j]);
            }
        }
        let max_distance = if self.precomputed {
            Some(storage.get_max())
        } else {
            None
        };
        Self {
            locations,
            storage,
            precomputed: true,
            rounded: self.rounded,
            max_distance,
        }
    }
}

const CORRELATION_LIMIT: usize = 200;

#[derive(Debug, Clone)]
pub struct CorrelationMatrix {
    storage: Matrix<usize>,
    width: usize,
}

impl CorrelationMatrix {
    pub fn new(distance_matrix: &DistanceMatrix) -> Self {
        let size = distance_matrix.size();
        let width = CORRELATION_LIMIT.min(size - 2);
        let mut matrix: Matrix<usize> = Matrix::new(size, width);
        for i in 0..size {
            distance_matrix
                .get_vec(i, 0, size)
                .iter()
                .enumerate()
                .filter(|&(j, _)| j > 0 && j != i)
                .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .take(width)
                .map(|(index, _)| index)
                .enumerate()
                .for_each(|(number, index)| {
                    matrix.set(i, number, index);
                });
        }
        Self {
            storage: matrix,
            width,
        }
    }

    pub fn get(&self, index: usize) -> &[usize] {
        self.slice(index, 0, self.width)
    }

    pub fn top_slice(&self, index: usize, number: usize) -> &[usize] {
        self.slice(index, 0, number)
    }

    pub fn top_slice_mut(&self, index: usize, number: usize) -> &mut [usize] {
        self.storage.slice_mut(index, 0, number)
    }

    fn slice(&self, row: usize, start: usize, number: usize) -> &[usize] {
        self.storage.slice(row, start, number)
    }
}

#[derive(Debug)]
pub struct MatrixProvider {
    pub distance: DistanceMatrix,
    pub correlation: CorrelationMatrix,
}

impl MatrixProvider {
    pub fn new(problem: &Problem, config: &Config, input_matrix: Option<Vec<Vec<f64>>>) -> Self {
        let rounded: bool = config.round_distances;
        let locations = problem.nodes.iter().map(|node| node.coord).collect();
        let precompute: bool =
            problem.nodes.len() - 1 < config.precompute_distance_size_limit as usize;
        let mut distance_builder = DistanceMatrixBuilder::new()
            .locations(locations)
            .precompute(precompute)
            .rounded(rounded);

        if let Some(input) = input_matrix {
            distance_builder = distance_builder.input(input);
        }

        let distance = distance_builder.build();

        let correlation = CorrelationMatrix::new(&distance);

        Self {
            distance,
            correlation,
        }
    }

    pub fn from_mapping(&self, mapping: &[usize]) -> Self {
        let distance = self.distance.from_mapping(mapping);
        let correlation = CorrelationMatrix::new(&distance);
        Self {
            distance,
            correlation,
        }
    }
}
