use polars::prelude::*;
use std::ops::RangeInclusive;

/// Extension methods for [`Column`]
pub trait ColumnExt {
    fn mode(&self) -> ModeSeries;
}

impl ColumnExt for Column {
    fn mode(&self) -> ModeSeries {
        ModeSeries::new(self).expect(r#"Expected "Mode" column"#)
    }
}

/// Mode series
#[derive(Clone)]
pub struct ModeSeries {
    pub onset_temperature: Series,
    pub temperature_step: Series,
}

impl ModeSeries {
    pub fn new(column: &Column) -> PolarsResult<Self> {
        let onset_temperature = column.struct_()?.field_by_name("OnsetTemperature")?;
        let temperature_step = column.struct_()?.field_by_name("TemperatureStep")?;
        Ok(Self {
            onset_temperature,
            temperature_step,
        })
    }

    pub fn onset_temperature_range(&self) -> RangeInclusive<f64> {
        if let Ok(onset_temperatures) = self.onset_temperature.f64() {
            if let Some((min, max)) = onset_temperatures.min_max() {
                return min..=max;
            }
        }
        0.0..=0.0
    }

    pub fn temperature_step_range(&self) -> RangeInclusive<f64> {
        if let Ok(temperature_steps) = self.temperature_step.f64() {
            if let Some((min, max)) = temperature_steps.min_max() {
                return min..=max;
            }
        }
        0.0..=0.0
    }

    pub fn onset_temperature(&self) -> PolarsResult<OnsetTemperature> {
        OnsetTemperature::new(&self.onset_temperature)
    }

    pub fn temperature_step(&self) -> PolarsResult<TemperatureStep> {
        TemperatureStep::new(&self.temperature_step)
    }
}

/// Onset temperature
#[derive(Clone)]
pub struct OnsetTemperature<'a>(&'a Float64Chunked);

impl<'a> OnsetTemperature<'a> {
    pub fn new(series: &'a Series) -> PolarsResult<Self> {
        Ok(Self(series.f64()?))
    }
}

impl OnsetTemperature<'_> {
    pub fn range(&self) -> RangeInclusive<f64> {
        if let Some((min, max)) = self.0.min_max() {
            return min..=max;
        }
        0.0..=0.0
    }

    pub fn unique(&self) -> Float64Chunked {
        self.0.unique().unwrap()
    }
}

/// Temperature step
#[derive(Clone)]
pub struct TemperatureStep<'a>(&'a Float64Chunked);

impl<'a> TemperatureStep<'a> {
    pub fn new(series: &'a Series) -> PolarsResult<Self> {
        Ok(Self(series.f64()?))
    }
}

impl TemperatureStep<'_> {
    pub fn range(&self) -> RangeInclusive<f64> {
        if let Some((min, max)) = self.0.min_max() {
            return min..=max;
        }
        0.0..=0.0
    }

    pub fn unique(&self) -> Float64Chunked {
        self.0.unique().unwrap()
    }
}
