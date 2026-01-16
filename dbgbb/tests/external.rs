#[cfg(any(feature = "ndarray_15", feature = "ndarray_16", feature = "ndarray_17"))]
mod test_ndarray {
    #[cfg(feature = "ndarray_15")]
    use ndarray_15 as ndarray;
    #[cfg(feature = "ndarray_16")]
    use ndarray_16 as ndarray;
    #[cfg(feature = "ndarray_17")]
    use ndarray_17 as ndarray;

    use dbgbb::*;
    use ndarray::Array2;
    #[test]
    fn ndarray_integer() {
        let v: Vec<_> = (-128..128).map(|i| i as i32).collect();
        let ndarr = Array2::from_shape_vec((16, 16), v).unwrap();
        dbgbb!(ndarr);
        let ndarr_recv: Array2<i32> = dbgbb_read!("ndarr");
        assert_eq!(ndarr, ndarr_recv);
    }
}

#[cfg(any(feature = "nalgebra_33", feature = "nalgebra_34"))]
mod test_nalgebra {
    #[cfg(feature = "nalgebra_33")]
    use nalgebra_33 as nalgebra;
    #[cfg(feature = "nalgebra_34")]
    use nalgebra_34 as nalgebra;

    use dbgbb::*;
    use nalgebra::DMatrix;
    #[test]
    fn nalgebra_integer() {
        let v: Vec<_> = (-128..128).map(|i| i as i32).collect();
        let nalg = DMatrix::from_vec(16, 16, v);
        dbgbb!(nalg);
        let nalg_recv: DMatrix<i32> = dbgbb_read!("nalg");
        assert_eq!(nalg, nalg_recv);
    }
}
