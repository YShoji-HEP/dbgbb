#[cfg(feature = "ndarray_15")]
use ndarray_15 as ndarray;
#[cfg(feature = "ndarray_16")]
use ndarray_16 as ndarray;

use crate::Rename;
use ndarray::{Array, Dimension};

impl<T, D: Dimension> Rename for Array<T, D> {}
