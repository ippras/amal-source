use crate::special::data_frame::DataFrameExt;

use super::Settings;
use egui::Ui;
use egui_ext::color;
use egui_plot::{MarkerShape, Plot, Points};
use itertools::izip;
use lipid::fatty_acid::{
    display::{COMMON, DisplayWithOptions},
    polars::DataFrameExt as _,
};
use polars::prelude::*;
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
    pub(super) fn ui(&mut self, ui: &mut Ui) {
        if let Err(error) = self.try_ui(ui) {
            error!(%error);
        }
    }

    fn try_ui(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        // let mode = &self.data_frame["Mode"];
        let index = self.data_frame["Index"].u32()?;
        let fatty_acid = self.data_frame.fatty_acid();
        let mode = self.data_frame.mode();
        let retention_time = self.data_frame["RetentionTime"].f64()?;
        let ecl = self.data_frame["ECL"].f64()?;
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
            for (index, fatty_acid, retention_time, ecl) in
                izip!(index, fatty_acid, retention_time, ecl)
            {
                let mut points = Vec::new();
                if let Some((retention_time, ecl)) = retention_time.zip(ecl) {
                    points.push([retention_time, ecl]);
                }
                let mut points = Points::new(points)
                    .color(color(index.unwrap() as _))
                    .radius(3.0);
                if let Some(fatty_acid) = fatty_acid {
                    points = points.name(format!("{:#}", (&fatty_acid).display(COMMON)));
                    if fatty_acid.unsaturation() == 0 {
                        points = points.shape(MarkerShape::Square).filled(false);
                    }
                    if fatty_acid.unsaturation() == 0 {
                        points = points.shape(MarkerShape::Square).filled(false);
                    }
                }
                ui.points(points);
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
