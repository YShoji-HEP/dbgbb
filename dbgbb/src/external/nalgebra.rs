use crate::Rename;
use nalgebra::base::dimension::Dim;
use nalgebra::base::storage::RawStorage;
use nalgebra::base::Matrix;

impl<R: Dim, C: Dim, T, S: RawStorage<T, R, C>> Rename for Matrix<T, R, C, S> {}
