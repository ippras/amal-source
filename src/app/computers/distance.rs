use crate::app::panes::distance::settings::{Order, Settings, Sort};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Distance computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Distance computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<DataFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        // Join
        lazy_frame = lazy_frame
            .clone()
            .select([
                as_struct(vec![col("FattyAcid"), col("Mode")]).alias("From"),
                col("RetentionTime")
                    .struct_()
                    .field_by_name("Absolute")
                    .struct_()
                    .field_by_name("Mean")
                    .alias("FromTime"),
                col("ChainLength")
                    .struct_()
                    .field_by_name("ECL")
                    .alias("FromECL"),
            ])
            .with_row_index("LeftIndex", None)
            .join_builder()
            .with(
                lazy_frame
                    .select([
                        as_struct(vec![col("FattyAcid"), col("Mode")]).alias("To"),
                        col("RetentionTime")
                            .struct_()
                            .field_by_name("Absolute")
                            .struct_()
                            .field_by_name("Mean")
                            .alias("ToTime"),
                        col("ChainLength")
                            .struct_()
                            .field_by_name("ECL")
                            .alias("ToECL"),
                    ])
                    .with_row_index("RightIndex", None),
            )
            .join_where(vec![
                // Same modes
                col("From")
                    .struct_()
                    .field_by_name("Mode")
                    .struct_()
                    .field_by_name("OnsetTemperature")
                    .eq(col("To")
                        .struct_()
                        .field_by_name("Mode")
                        .struct_()
                        .field_by_name("OnsetTemperature"))
                    .and(
                        col("From")
                            .struct_()
                            .field_by_name("Mode")
                            .struct_()
                            .field_by_name("TemperatureStep")
                            .eq(col("To")
                                .struct_()
                                .field_by_name("Mode")
                                .struct_()
                                .field_by_name("TemperatureStep")),
                    ),
                // Fatty asids not equals combination
                col("LeftIndex").lt(col("RightIndex")),
            ]);
        // Cache
        lazy_frame = lazy_frame.cache().select([
            col("From").struct_().field_by_name("Mode").alias("Mode"),
            col("From")
                .struct_()
                .field_by_name("FattyAcid")
                .name()
                .keep(),
            col("To").struct_().field_by_name("FattyAcid").name().keep(),
            as_struct(vec![
                col("FromTime").alias("From"),
                col("ToTime").alias("To"),
                (col("ToTime") - col("FromTime"))
                    .over([col("From").struct_().field_by_name("Mode")])
                    .alias("Delta"),
            ])
            .alias("RetentionTime"),
            as_struct(vec![
                col("FromECL").alias("From"),
                col("ToECL").alias("To"),
                (col("ToECL") - col("FromECL"))
                    .over([col("From").struct_().field_by_name("Mode")])
                    .alias("Delta"),
            ])
            .alias("ECL"),
        ]);
        // Sort
        let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
        if key.settings.order == Order::Descending {
            sort_options = sort_options.with_order_descending(true);
        };
        lazy_frame = match key.settings.sort {
            Sort::Ecl => lazy_frame.sort_by_exprs(
                [col("ECL")
                    .struct_()
                    .field_by_name("Delta")
                    .abs()
                    .median()
                    .over([col("Mode")])],
                sort_options,
            ),
            Sort::Time => lazy_frame.sort_by_exprs(
                [col("RetentionTime")
                    .struct_()
                    .field_by_name("Delta")
                    .abs()
                    .median()
                    .over([col("Mode")])],
                sort_options,
            ),
        };
        // Index
        lazy_frame = lazy_frame.with_row_index("Index", None);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        self.try_compute(key).unwrap()
    }
}

/// Distance key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.filter_onset_temperature.hash(state);
        self.settings.filter_temperature_step.hash(state);
        self.settings.interpolation.hash(state);

        self.settings.filter.hash(state);
        self.settings.sort.hash(state);
        self.settings.order.hash(state);
    }
}
