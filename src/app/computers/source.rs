use crate::app::{
    MAX_TEMPERATURE,
    panes::source::settings::{Group, Kind, Order, Settings, Sort},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::{
    fatty_acid::{Kind as FattyAcidKind, polars::expr::FattyAcidExpr},
    prelude::*,
};
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Source computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Source computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<LazyFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        lazy_frame = lazy_frame
            .with_columns([
                // Retention time mean
                col("RetentionTime")
                    .list()
                    .mean()
                    .alias("RetentionTimeMean"),
                // Retention time standard deviation
                col("RetentionTime")
                    .list()
                    .std(key.settings.ddof)
                    .alias("RetentionTimeStandardDeviation"),
            ])
            .with_columns([
                // Relative retention time
                relative_time(key.settings)
                    .over(["Mode"])
                    .alias("RelativeRetentionTime"),
                // Delta retention time
                col("FattyAcid")
                    .fa()
                    .delta(col("RetentionTimeMean"))
                    .over(["Mode"])
                    .alias("DeltaRetentionTime"),
                // Temperature
                (col("Mode").struct_().field_by_name("OnsetTemperature")
                    + col("RetentionTimeMean")
                        * col("Mode").struct_().field_by_name("TemperatureStep"))
                .clip_max(lit(MAX_TEMPERATURE))
                .alias("Temperature"),
                // FCL
                col("FattyAcid")
                    .fa()
                    .fcl(
                        col("RetentionTimeMean"),
                        ChainLengthOptions::new().logarithmic(key.settings.logarithmic),
                    )
                    .over(["Mode"])
                    .alias("FCL"),
                // ECL
                col("FattyAcid")
                    .fa()
                    .ecl(
                        col("RetentionTimeMean"),
                        ChainLengthOptions::new().logarithmic(key.settings.logarithmic),
                    )
                    .over(["Mode"])
                    .alias("ECL"),
                // ECN
                col("FattyAcid").fa().ecn().alias("ECN"),
            ])
            .with_columns([
                // Slope
                col("FattyAcid")
                    .fa()
                    .slope(col("ECL"), col("RetentionTimeMean"))
                    .over(["Mode"])
                    .alias("Slope"),
            ])
            .select([
                col("Mode"),
                col("FattyAcid"),
                // Retention time
                as_struct(vec![
                    as_struct(vec![
                        col("RetentionTimeMean").alias("Mean"),
                        col("RetentionTimeStandardDeviation").alias("StandardDeviation"),
                        col("RetentionTime").alias("Values"),
                    ])
                    .alias("Absolute"),
                    col("RelativeRetentionTime").alias("Relative"),
                    col("DeltaRetentionTime").alias("Delta"),
                ])
                .alias("RetentionTime"),
                // Temperature
                col("Temperature"),
                // Chain length
                as_struct(vec![col("ECL"), col("FCL"), col("ECN")]).alias("ChainLength"),
                // Mass
                as_struct(vec![
                    col("FattyAcid").fa().mass(FattyAcidKind::Rco).alias("RCO"),
                    col("FattyAcid")
                        .fa()
                        .mass(FattyAcidKind::Rcoo)
                        .alias("RCOO"),
                    col("FattyAcid")
                        .fa()
                        .mass(FattyAcidKind::Rcooh)
                        .alias("RCOOH"),
                    col("FattyAcid")
                        .fa()
                        .mass(FattyAcidKind::Rcooch3)
                        .alias("RCOOCH3"),
                ])
                .alias("Mass"),
                // Derivative
                as_struct(vec![
                    col("Slope"),
                    col("Slope").arctan().degrees().alias("Angle"),
                ])
                .alias("Derivative"),
            ]);
        // Filter
        if let Some(onset_temperature) = key.settings.filter.mode.onset_temperature {
            lazy_frame = lazy_frame.filter(
                col("Mode")
                    .struct_()
                    .field_by_name("OnsetTemperature")
                    .eq(lit(onset_temperature)),
            );
        }
        if let Some(temperature_step) = key.settings.filter.mode.temperature_step {
            lazy_frame = lazy_frame.filter(
                col("Mode")
                    .struct_()
                    .field_by_name("TemperatureStep")
                    .eq(lit(temperature_step)),
            );
        }
        if !key.settings.filter.fatty_acids.is_empty() {
            let mut expr = lit(false);
            for fatty_acid in &key.settings.filter.fatty_acids {
                expr = expr.or(col("FattyAcid").fa().equal(fatty_acid));
            }
            lazy_frame = lazy_frame.filter(expr);
        }
        // Interpolate
        // Sort
        let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
        if key.settings.order == Order::Descending {
            sort_options = sort_options.with_order_descending(true);
        };
        lazy_frame = match key.settings.sort {
            Sort::FattyAcid => lazy_frame.sort_by_fatty_acids(sort_options),
            Sort::Time => lazy_frame.sort_by_time(sort_options),
        };
        Ok(lazy_frame)
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        let mut lazy_frame = self.try_compute(key).expect("compute source");
        lazy_frame = match key.settings.kind {
            Kind::Plot => {
                lazy_frame = lazy_frame.select([
                    col("Mode").struct_().field_by_name("OnsetTemperature"),
                    col("Mode").struct_().field_by_name("TemperatureStep"),
                    col("FattyAcid"),
                    col("RetentionTime")
                        .struct_()
                        .field_by_name("Absolute")
                        .struct_()
                        .field_by_name("Mean")
                        .alias("RetentionTime"),
                    col("ChainLength")
                        .struct_()
                        .field_by_name("ECL")
                        .alias("ECL"),
                ]);
                println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());
                // let lazy_frame = lazy_frame.rank()
                let lazy_frame =
                    match key.settings.group {
                        Group::FattyAcid => lazy_frame
                            .group_by([col("FattyAcid"), col("OnsetTemperature")])
                            .agg([col("TemperatureStep"), col("RetentionTime"), col("ECL")])
                            .group_by([col("FattyAcid")])
                            .agg([
                                col("OnsetTemperature"),
                                col("TemperatureStep"),
                                col("RetentionTime"),
                                col("ECL"),
                            ])
                            .sort(["FattyAcid"], Default::default()),
                        Group::OnsetTemperature => lazy_frame
                            .group_by([col("OnsetTemperature")])
                            .agg([as_struct(vec![
                                col("TemperatureStep"),
                                col("FattyAcid"),
                                col("RetentionTime"),
                                col("ECL"),
                            ])
                            .alias("Value")]),
                        Group::TemperatureStep => lazy_frame
                            .group_by([col("TemperatureStep")])
                            .agg([as_struct(vec![
                                col("OnsetTemperature"),
                                col("FattyAcid"),
                                col("RetentionTime"),
                                col("ECL"),
                            ])
                            .alias("Value")]),
                    };
                println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());
                lazy_frame
            }
            Kind::Table => lazy_frame,
        };
        // Index
        lazy_frame = lazy_frame.with_row_index("Index", None);
        lazy_frame.collect().expect("collect source")
    }
}

/// Source key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.kind.hash(state);
        self.settings.ddof.hash(state);
        self.settings.logarithmic.hash(state);
        self.settings.relative.hash(state);
        self.settings.filter.hash(state);
        self.settings.sort.hash(state);
        self.settings.order.hash(state);
        self.settings.group.hash(state);
    }
}

/// Extension methods for [`LazyFrame`]
trait LazyFrameExt {
    fn sort_by_fatty_acids(self, sort_options: SortMultipleOptions) -> LazyFrame;

    fn sort_by_time(self, sort_options: SortMultipleOptions) -> LazyFrame;
}

impl LazyFrameExt for LazyFrame {
    fn sort_by_fatty_acids(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort_by_exprs([col("Mode"), col("FattyAcid")], sort_options)
    }

    fn sort_by_time(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort(["Mode"], sort_options.clone()).select([all()
            .sort_by(
                &[
                    col("ChainLength").struct_().field_by_name("ECL"),
                    col("RetentionTime")
                        .struct_()
                        .field_by_name("Absolute")
                        .struct_()
                        .field_by_name("Mean"),
                ],
                sort_options,
            )
            .over([col("Mode")])])
    }
}

fn relative_time(settings: &Settings) -> Expr {
    match &settings.relative {
        Some(relative) => {
            col("RetentionTimeMean")
                / col("RetentionTimeMean")
                    .filter(col("FattyAcid").fa().equal(relative))
                    .first()
        }
        None => lit(f64::NAN),
    }
}

/// Saturated
pub trait Saturated {
    /// Delta
    fn delta(self, expr: Expr) -> Expr;

    /// Slope
    fn slope(self, dividend: Expr, divisor: Expr) -> Expr;

    /// Backward saturated
    fn backward(self, expr: Expr) -> Expr;

    /// Forward saturated
    fn forward(self, expr: Expr) -> Expr;
}

impl Saturated for FattyAcidExpr {
    fn delta(self, expr: Expr) -> Expr {
        self.clone().backward(expr.clone()) - self.clone().forward(expr)
    }

    fn slope(self, dividend: Expr, divisor: Expr) -> Expr {
        self.clone().delta(dividend) / self.clone().delta(divisor)
    }

    fn backward(self, expr: Expr) -> Expr {
        self.clone().saturated_or_null(expr).backward_fill(None)
    }

    fn forward(self, expr: Expr) -> Expr {
        self.clone().saturated_or_null(expr).forward_fill(None)
    }
}
