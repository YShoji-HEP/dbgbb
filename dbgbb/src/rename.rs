use num_complex::Complex;

/// Rename the variable.
pub trait Rename {
    /// Returns the name of the variable if set.
    fn get_name(&self) -> Option<String> {
        None
    }
    /// Set the name of the variable.
    fn rename(&self, name: &str) -> Renamed<Self>
    where
        Self: Sized + Clone,
    {
        Renamed::<Self> {
            name: name.to_string(),
            data: self.clone(),
        }
    }
}

macro_rules! impl_rename {
    ($($ty:ty),*) => {
        $(
            impl Rename for $ty {}
        )*
    }
}

impl_rename!(
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    Complex<f32>,
    Complex<f64>,
    String,
    &str
);

impl<T: Clone> Rename for Vec<T> {}

impl<T: Clone, const N: usize> Rename for [T; N] {}

pub struct Renamed<T> {
    name: String,
    data: T,
}

impl<T> Rename for Renamed<T> {
    fn get_name(&self) -> Option<String> {
        Some(self.name.clone())
    }
}

impl<T: Clone> Renamed<T> {
    pub fn clone(&self) -> T {
        self.data.clone()
    }
}
