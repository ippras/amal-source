use super::Settings;
use crate::app::panes::widgets::float::FloatValue;
use egui::{Frame, Id, Margin, TextStyle, TextWrapMode, Ui, Vec2, vec2};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use lipid::fatty_acid::{
    display::{COMMON, DisplayWithOptions as _},
    polars::ColumnExt,
};
use polars::prelude::*;

const INDEX: usize = 0;
const MODE: usize = 1;
const FROM: usize = 2;
const TO: usize = 3;
const TIME: usize = 4;
const ECL: usize = 5;

const MARGIN: Vec2 = vec2(4.0, 0.0);

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
        let num_columns = self.data_frame.width();
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky)
            .headers([HeaderRow::new(height)])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            (0, INDEX) => {
                ui.heading("Index");
            }
            (0, MODE) => {
                ui.heading("Mode");
            }
            (0, FROM) => {
                ui.heading("From");
            }
            (0, TO) => {
                ui.heading("To");
            }
            (0, TIME) => {
                ui.heading("Δ Time");
            }
            (0, ECL) => {
                ui.heading("Δ ECL");
            }
            _ => {} // _ => unreachable!(),
        }
    }

    fn body_cell_content_ui(&mut self, ui: &mut Ui, row: usize, col: usize) -> PolarsResult<()> {
        match (row, col) {
            (row, INDEX) => {
                let indices = self.data_frame["Index"].u32()?;
                let value = indices.get(row).unwrap();
                ui.label(value.to_string());
            }
            (row, MODE) => {
                let mode = self.data_frame["Mode"].struct_()?;
                let onset_temperature = mode.field_by_name("OnsetTemperature")?;
                let temperature_step = mode.field_by_name("TemperatureStep")?;
                ui.label(format!(
                    "{}/{}",
                    onset_temperature.str_value(row)?,
                    temperature_step.str_value(row)?
                ));
            }
            (row, FROM) => {
                let fatty_acids = self.data_frame["From"].fatty_acid();
                let fatty_acid = fatty_acids.get(row)?.unwrap();
                ui.label(format!("{:#}", fatty_acid.display(COMMON)));
                // .on_hover_text(fatty_acid.label());
            }
            (row, TO) => {
                let fatty_acids = self.data_frame["To"].fatty_acid();
                let fatty_acid = fatty_acids.get(row)?.unwrap();
                ui.label(format!("{:#}", fatty_acid.display(COMMON)));
                // .on_hover_text(fatty_acid.label());
            }
            (row, TIME) => {
                let retention_time = self.data_frame["RetentionTime"].struct_().unwrap();
                let delta = retention_time.field_by_name("Delta").unwrap();
                ui.add(
                    FloatValue::new(delta.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                )
                .on_hover_ui(|ui| {
                    let from = retention_time.field_by_name("From").unwrap();
                    let to = retention_time.field_by_name("To").unwrap();
                    ui.horizontal(|ui| {
                        ui.label(to.str_value(row).unwrap());
                        ui.label("-");
                        ui.label(from.str_value(row).unwrap());
                    });
                });
            }
            (row, ECL) => {
                let ecl = self.data_frame["ECL"].struct_().unwrap();
                let delta = ecl.field_by_name("Delta").unwrap();
                ui.add(
                    FloatValue::new(delta.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                )
                .on_hover_ui(|ui| {
                    let from = ecl.field_by_name("From").unwrap();
                    let to = ecl.field_by_name("To").unwrap();
                    ui.horizontal(|ui| {
                        ui.label(to.str_value(row).unwrap());
                        ui.label("-");
                        ui.label(from.str_value(row).unwrap());
                    });
                });
            }
            (row, column) => {
                let value = self.data_frame[column].get(row).unwrap();
                ui.label(value.to_string());
            }
        }
        Ok(())
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.group_index)
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 0 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr)
                    .unwrap()
            });
    }
}
