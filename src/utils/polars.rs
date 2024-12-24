use egui::WidgetText;
use polars::{chunked_array::ChunkedArray, datatypes::PolarsDataType, series::Series};
use std::fmt::{Display, Formatter, Result};

// /// Extension methods for [`Series`]
// trait SeriesExt {
//     fn display(&self, fmt: impl Fn(T) -> U) -> FirstAndLast<T, U, impl Fn(T::Physical<'_>) -> U>;
// }

// impl<T: PolarsDataType, U: Display> ChunkedArrayExt<T, U> for &Series {
//     fn display(&self, fmt: impl Fn(T) -> U) -> FirstAndLast<T, U, impl Fn(T::Physical<'_>) -> U> {
//         FirstAndLast {
//             chunked_array: self,
//             fmt,
//         }
//     }
// }

// struct DisplayList {
//     chunked_array: self,
//     fmt,
// }

/// Extension methods for [`ChunkedArray`]
pub trait ChunkedArrayExt<T: PolarsDataType, U: Display> {
    fn display(
        &self,
        fmt: impl Fn(T::Physical<'_>) -> U,
    ) -> FirstAndLast<T, U, impl Fn(T::Physical<'_>) -> U>;
}

impl<T: PolarsDataType, U: Display> ChunkedArrayExt<T, U> for &ChunkedArray<T> {
    fn display(
        &self,
        fmt: impl Fn(T::Physical<'_>) -> U,
    ) -> FirstAndLast<T, U, impl Fn(T::Physical<'_>) -> U> {
        FirstAndLast {
            chunked_array: self,
            fmt,
        }
    }
}

/// Display first and last
pub struct FirstAndLast<'a, T: PolarsDataType, U: Display, F: Fn(T::Physical<'a>) -> U> {
    chunked_array: &'a ChunkedArray<T>,
    fmt: F,
}

impl<T: PolarsDataType, U: Display, F: Fn(T::Physical<'_>) -> U> Display
    for FirstAndLast<'_, T, U, F>
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "[")?;
        if !self.chunked_array.is_empty() {
            let first = self.chunked_array.get(0).unwrap();
            Display::fmt(&(self.fmt)(first), f)?;
            if self.chunked_array.len() > 1 {
                write!(f, ", ")?;
                if self.chunked_array.len() > 2 {
                    write!(f, "… ")?;
                }
                let last = self
                    .chunked_array
                    .get(self.chunked_array.len() - 1)
                    .unwrap();
                Display::fmt(&(self.fmt)(last), f)?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<T: PolarsDataType, U: Display, F: Fn(T::Physical<'_>) -> U> From<FirstAndLast<'_, T, U, F>>
    for WidgetText
{
    fn from(value: FirstAndLast<T, U, F>) -> Self {
        value.to_string().into()
    }
}
