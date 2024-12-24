use super::column::mode::{ColumnExt, ModeSeries};
use polars::frame::DataFrame;

/// Extension methods for [`DataFrame`]
pub trait DataFrameExt {
    fn mode(&self) -> ModeSeries;
}

impl DataFrameExt for DataFrame {
    fn mode(&self) -> ModeSeries {
        self["Mode"].mode()
    }
}
