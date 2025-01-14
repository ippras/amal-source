use super::Settings;
use crate::special::data_frame::DataFrameExt;
use egui::{Ui, util::hash};
use egui_ext::color;
use egui_plot::{Line, LineStyle, MarkerShape, Plot, PlotItem, Points};
use itertools::izip;
use lipid::fatty_acid::{
    FattyAcidExt,
    display::{COMMON, DisplayWithOptions},
    polars::DataFrameExt as _,
};
use polars::prelude::*;
use std::iter::zip;
use tracing::error;

/// Plot view
#[derive(Clone, Debug)]
pub(crate) struct PlotView<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl<'a> PlotView<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }
}

impl PlotView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        if let Err(error) = self.try_show(ui) {
            error!(%error);
        }
    }

    fn try_show(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        // let mode = &self.data_frame["Mode"];
        let index = self.data_frame["Index"].u32()?;
        let fatty_acid = self.data_frame.fatty_acid();
        let onset_temperature = self.data_frame["OnsetTemperature"].list()?;
        let temperature_step = self.data_frame["TemperatureStep"].list()?;
        let retention_time = self.data_frame["RetentionTime"].list()?;
        let ecl = self.data_frame["ECL"].list()?;
        let mut plot = Plot::new("plot")
            // .allow_drag(context.settings.visualization.drag)
            // .allow_scroll(context.settings.visualization.scroll)
            ;
        if self.settings.legend {
            plot = plot.legend(Default::default());
        }
        // let scale = plot.transform.dvalue_dpos();
        // let x_decimals = ((-scale[0].abs().log10()).ceil().at_least(0.0) as usize).clamp(1, 6);
        // let y_decimals = ((-scale[1].abs().log10()).ceil().at_least(0.0) as usize).clamp(1, 6);
        plot = plot.label_formatter(|name, value| {
            let name = if !name.is_empty() {
                format!("{name}\n")
            } else {
                String::new()
            };
            format!("{name}x = {}\ny = {}", value.x, value.y)
            // format!(
            //     "{}x = {:.*}\ny = {:.*}",
            //     name, x_decimals, value.x, y_decimals, value.y
            // )
        });
        plot.show(ui, |ui| -> PolarsResult<()> {
            for (index, fatty_acid, onset_temperature, temperature_step, retention_time, ecl) in izip!(index, fatty_acid, onset_temperature, temperature_step, retention_time, ecl) {
                let Some(index) = index else {
                    polars_bail!(NoData: "Index");
                };
                let Some(fatty_acid) = fatty_acid else {
                    polars_bail!(NoData: "FattyAcid");
                };
                let Some(onset_temperature) = onset_temperature else {
                    polars_bail!(NoData: "OnsetTemperature");
                };
                let Some(temperature_step) = temperature_step else {
                    polars_bail!(NoData: "TemperatureStep");
                };
                let Some(retention_time) = retention_time else {
                    polars_bail!(NoData: "RetentionTime");
                };
                let Some(ecl) = ecl else {
                    polars_bail!(NoData: "ECL");
                };
                for (onset_temperature, temperature_step, retention_time, ecl) in izip!(onset_temperature.f64()?, temperature_step.list()?, retention_time.list()?, ecl.list()?) {
                    let Some(onset_temperature) = onset_temperature else {
                        polars_bail!(NoData: "OnsetTemperature");
                    };
                    let Some(temperature_step) = temperature_step else {
                        polars_bail!(NoData: "TemperatureStep");
                    };
                    let Some(retention_time) = retention_time else {
                        polars_bail!(NoData: "RetentionTime");
                    };
                    let Some(ecl) = ecl else {
                        polars_bail!(NoData: "ECL");
                    };
                    let mut points = Vec::new();
                    for (temperature_step, retention_time, ecl) in izip!(temperature_step.f64()?, retention_time.f64()?, ecl.f64()?) {
                        let Some(temperature_step) = temperature_step else {
                            polars_bail!(NoData: "TemperatureStep");
                        };
                        if let Some((retention_time, ecl)) = retention_time.zip(ecl) {
                            points.push([retention_time, ecl]);
                        }
                    }
                    // Line
                    let mut line = Line::new(points.clone()).name(format!("{:#} {onset_temperature}", (&fatty_acid).display(COMMON)));
                    if fatty_acid.unsaturation() != 0 {
                        line = line.style(LineStyle::Dashed { length: 16.0 });
                    }
                    ui.line(line);
                    // Points
                    let mut points = Points::new(points)
                        .name(format!("{:#} {temperature_step}", (&fatty_acid).display(COMMON)))
                        .color(color(onset_temperature as _))
                        // .name(onset_temperature)
                        // .color(color(onset_temperature as _))
                        .radius(3.0);
                    if fatty_acid.unsaturation() == 0 {
                        points = points.shape(MarkerShape::Square);
                    }
                    ui.points(points);
                }

                // if let Some(fatty_acid) = fatty_acid {
                //     points = points.name(format!("{:#}", (&fatty_acid).display(COMMON)));
                //     if fatty_acid.unsaturation() == 0 {
                //         points = points.shape(MarkerShape::Square).filled(false);
                //     }
                //     if fatty_acid.unsaturation() == 0 {
                //         points = points.shape(MarkerShape::Square).filled(false);
                //     }
                // }
                // let mut points = Points::new(points);
                // 
            }
            // let mut offsets = HashMap::new();
            // for (key, values) in visualized {
            //     // Bars
            //     let mut offset = 0.0;
            //     let x = key.into_inner();
            //     for (name, value) in values {
            //         let mut y = value;
            //         if percent {
            //             y *= 100.0;
            //         }
            //         let bar = Bar::new(x, y).name(name).base_offset(offset);
            //         let chart = BarChart::new(vec![bar])
            //             .width(context.settings.visualization.width)
            //             .name(x)
            //             .color(color(x as _));
            //         ui.bar_chart(chart);
            //         offset += y;
            //     }
            //     // // Text
            //     // if context.settings.visualization.text.show
            //     //     && offset >= context.settings.visualization.text.min
            //     // {
            //     //     let y = offset;
            //     //     let text = Text::new(
            //     //         PlotPoint::new(x, y),
            //     //         RichText::new(format!("{y:.p$}"))
            //     //             .size(context.settings.visualization.text.size)
            //     //             .heading(),
            //     //     )
            //     //     .name(x)
            //     //     .color(color(x as _))
            //     //     .anchor(Align2::CENTER_BOTTOM);
            //     //     ui.text(text);
            //     // }
            // }
            Ok(())
        });
        Ok(())
    }
}
