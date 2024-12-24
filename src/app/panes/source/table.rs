use super::Settings;
use crate::app::panes::widgets::float::FloatValue;
use egui::{Frame, Grid, Id, Margin, TextStyle, TextWrapMode, Ui, Vec2, vec2};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use lipid::fatty_acid::{
    display::{COMMON, DisplayWithOptions as _},
    polars::ColumnExt,
};
use polars::prelude::*;
use std::ops::Range;

const MARGIN: Vec2 = vec2(4.0, 0.0);

const ID: Range<usize> = 0..3;
const RETENTION_TIME: Range<usize> = ID.end..ID.end + 3;
const TEMPERATURE: Range<usize> = RETENTION_TIME.end..RETENTION_TIME.end + 1;
const CHAIN_LENGTH: Range<usize> = TEMPERATURE.end..TEMPERATURE.end + 3;
const MASS: Range<usize> = CHAIN_LENGTH.end..CHAIN_LENGTH.end + 1;
const DERIVATIVE: Range<usize> = MASS.end..MASS.end + 2;
const LEN: usize = DERIVATIVE.end;

const TOP: &[Range<usize>] = &[
    ID,
    RETENTION_TIME,
    TEMPERATURE,
    CHAIN_LENGTH,
    MASS,
    DERIVATIVE,
];

/// Table view
#[derive(Clone, Debug)]
pub(crate) struct TableView<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl<'a> TableView<'a> {
    pub(crate) const fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }
}

impl TableView<'_> {
    pub(super) fn ui(&mut self, ui: &mut Ui) {
        ui.visuals_mut().collapsing_header_frame = true;
        let id_salt = Id::new("Table");
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as _;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky)
            .headers([
                HeaderRow {
                    height,
                    groups: TOP.to_vec(),
                },
                HeaderRow::new(height),
            ])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, ID) => {
                ui.heading("ID");
            }
            (0, RETENTION_TIME) => {
                ui.heading("Retention time");
            }
            (0, TEMPERATURE) => {
                ui.heading("Temperature");
            }
            (0, CHAIN_LENGTH) => {
                ui.heading("Chain length");
            }
            (0, MASS) => {
                ui.heading("Mass");
            }
            (0, DERIVATIVE) => {
                ui.heading("Derivative");
            }
            // Bottom
            (1, id::INDEX) => {
                ui.heading("Index");
            }
            (1, id::MODE) => {
                ui.heading("Mode");
            }
            (1, id::FA) => {
                ui.heading("Fatty acid");
            }
            (1, retention_time::ABSOLUTE) => {
                ui.heading("Absolute");
            }
            (1, retention_time::RELATIVE) => {
                ui.heading("Relative");
            }
            (1, retention_time::DELTA) => {
                ui.heading("Delta");
            }
            (1, chain_length::ECL) => {
                ui.heading("ECL");
            }
            (1, chain_length::FCL) => {
                ui.heading("FCL");
            }
            (1, chain_length::ECN) => {
                ui.heading("ECN");
            }
            (1, derivative::SLOPE) => {
                ui.heading("Slope");
            }
            (1, derivative::ANGLE) => {
                ui.heading("Angle");
            }
            _ => {}
        }
    }

    fn body_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        match (row, column) {
            (row, id::INDEX) => {
                let index = self.data_frame["Index"].u32().unwrap();
                let value = index.get(row).unwrap();
                ui.label(value.to_string());
            }
            (row, id::MODE) => {
                let mode = self.data_frame["Mode"].struct_().unwrap();
                let onset_temperature = mode.field_by_name("OnsetTemperature").unwrap();
                let temperature_step = mode.field_by_name("TemperatureStep").unwrap();
                ui.label(format!(
                    "{}/{}",
                    onset_temperature.str_value(row).unwrap(),
                    temperature_step.str_value(row).unwrap()
                ));
            }
            (row, id::FA) => {
                let fatty_acids = self.data_frame["FattyAcid"].fatty_acid();
                let fatty_acid = fatty_acids.get(row).unwrap().unwrap();
                ui.label(format!("{:#}", fatty_acid.display(COMMON)));
                // .on_hover_text(fatty_acid.label());
            }
            (row, retention_time::ABSOLUTE) => {
                let retention_time = self.data_frame["RetentionTime"].struct_().unwrap();
                let absolute = retention_time.field_by_name("Absolute").unwrap();
                let absolute = absolute.struct_().unwrap();
                let mean = absolute.field_by_name("Mean").unwrap();
                ui.add(
                    FloatValue::new(mean.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision)),
                )
                .on_hover_ui(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let standard_deviation = absolute.field_by_name("StandardDeviation").unwrap();
                    ui.horizontal(|ui| {
                        ui.label(mean.str_value(row).unwrap());
                        ui.label("±");
                        ui.label(standard_deviation.str_value(row).unwrap());
                    });
                })
                .on_hover_ui(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.heading("Repeats");
                    let values = absolute
                        .field_by_name("Values")
                        .unwrap()
                        .list()
                        .unwrap()
                        .get_as_series(row)
                        .unwrap();
                    ui.vertical(|ui| {
                        for value in values.iter() {
                            ui.label(value.to_string());
                        }
                    });
                });
            }
            (row, retention_time::RELATIVE) => {
                let retention_time = self.data_frame["RetentionTime"].struct_().unwrap();
                let relative = retention_time.field_by_name("Relative").unwrap();
                ui.add(
                    FloatValue::new(relative.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, retention_time::DELTA) => {
                let retention_time = self.data_frame["RetentionTime"].struct_().unwrap();
                let delta = retention_time.field_by_name("Delta").unwrap();
                ui.add(
                    FloatValue::new(delta.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, TEMPERATURE) => {
                let temperature = &self.data_frame["Temperature"];
                ui.add(
                    FloatValue::new(temperature.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, chain_length::ECL) => {
                let chain_length = self.data_frame["ChainLength"].struct_().unwrap();
                let ecl = chain_length.field_by_name("ECL").unwrap();
                ui.add(
                    FloatValue::new(ecl.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, chain_length::FCL) => {
                let chain_length = self.data_frame["ChainLength"].struct_().unwrap();
                let fcl = chain_length.field_by_name("FCL").unwrap();
                ui.add(
                    FloatValue::new(fcl.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, chain_length::ECN) => {
                let chain_length = self.data_frame["ChainLength"].struct_().unwrap();
                let ecn = chain_length.field_by_name("ECN").unwrap();
                ui.label(ecn.str_value(row).unwrap());
            }
            (row, MASS) => {
                let mass = self.data_frame["Mass"].struct_().unwrap();
                let rcooch3 = mass.field_by_name("RCOOCH3").unwrap();
                ui.add(
                    FloatValue::new(rcooch3.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision)),
                )
                .on_hover_ui(|ui| {
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        ui.label("[RCO]+");
                        let rcoo = mass.field_by_name("RCO").unwrap();
                        ui.label(rcoo.str_value(row).unwrap());
                        ui.end_row();

                        ui.label("[RCOO]-");
                        let rcoo = mass.field_by_name("RCOO").unwrap();
                        ui.label(rcoo.str_value(row).unwrap());
                        ui.end_row();

                        ui.label("RCOOH");
                        let rcooh = mass.field_by_name("RCOOH").unwrap();
                        ui.label(rcooh.str_value(row).unwrap());
                        ui.end_row();

                        ui.label("RCOOCH3");
                        ui.label(rcooch3.str_value(row).unwrap());
                    });
                });
            }
            (row, derivative::SLOPE) => {
                let derivative = self.data_frame["Derivative"].struct_().unwrap();
                let slope = derivative.field_by_name("Slope").unwrap();
                ui.add(
                    FloatValue::new(slope.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, derivative::ANGLE) => {
                let derivative = self.data_frame["Derivative"].struct_().unwrap();
                let angle = derivative.field_by_name("Angle").unwrap();
                ui.add(
                    FloatValue::new(angle.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            _ => unreachable!(),
            // _ => {}
        }
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.clone())
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 1 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1)
            });
    }
}

mod id {
    use super::*;

    pub(super) const INDEX: Range<usize> = ID.start..ID.start + 1;
    pub(super) const MODE: Range<usize> = INDEX.end..INDEX.end + 1;
    pub(super) const FA: Range<usize> = MODE.end..MODE.end + 1;
}

mod retention_time {
    use super::*;

    pub(super) const ABSOLUTE: Range<usize> = RETENTION_TIME.start..RETENTION_TIME.start + 1;
    pub(super) const RELATIVE: Range<usize> = ABSOLUTE.end..ABSOLUTE.end + 1;
    pub(super) const DELTA: Range<usize> = RELATIVE.end..RELATIVE.end + 1;
}

mod chain_length {
    use super::*;

    pub(super) const ECL: Range<usize> = CHAIN_LENGTH.start..CHAIN_LENGTH.start + 1;
    pub(super) const FCL: Range<usize> = ECL.end..ECL.end + 1;
    pub(super) const ECN: Range<usize> = FCL.end..FCL.end + 1;
}

mod derivative {
    use super::*;

    pub(super) const SLOPE: Range<usize> = DERIVATIVE.start..DERIVATIVE.start + 1;
    pub(super) const ANGLE: Range<usize> = SLOPE.end..SLOPE.end + 1;
}
