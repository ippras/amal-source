use crate::{
    app::{MAX_PRECISION, localize, text::Text},
    special::{column::mode::ColumnExt as _, data_frame::DataFrameExt as _},
};
use egui::{ComboBox, Grid, RichText, Slider, Ui, emath::Float};
use egui_ext::LabeledSeparator;
use egui_phosphor::regular::TRASH;
use lipid::fatty_acid::{
    FattyAcid,
    display::{COMMON, DisplayWithOptions},
    polars::{DataFrameExt as _, column::ColumnExt as _},
};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) kind: Kind,
    pub(crate) ddof: u8,
    pub(crate) logarithmic: bool,
    pub(crate) relative: Option<FattyAcid>,
    pub(crate) filter: Filter,
    pub(crate) sort: Sort,
    pub(crate) order: Order,

    pub(crate) group: Group,
    pub(crate) legend: bool,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            precision: 2,
            resizable: false,
            sticky: 1,
            truncate: false,

            kind: Kind::Table,
            ddof: 1,
            logarithmic: false,
            relative: None,
            filter: Filter::new(),
            sort: Sort::Time,
            order: Order::Ascending,

            group: Group::FattyAcid,
            legend: true,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, data_frame: &DataFrame) -> PolarsResult<()> {
        Grid::new("calculation")
            .show(ui, |ui| {
                // Precision floats
                ui.label(localize!("precision"));
                ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
                ui.end_row();

                // Sticky columns
                ui.label(localize!("sticky"));
                ui.add(Slider::new(&mut self.sticky, 0..=data_frame.width()));
                ui.end_row();

                // Truncate titles
                ui.label(localize!("truncate"));
                ui.checkbox(&mut self.truncate, "");
                ui.end_row();

                // Calculate
                ui.separator();
                ui.labeled_separator(RichText::new("Calculate").heading());
                ui.end_row();

                // Relative
                ui.label("Relative").on_hover_text("Relative fatty acid");
                ui.horizontal(|ui| {
                    let selected_text = self
                        .relative
                        .as_ref()
                        .map(|relative| relative.display(COMMON).to_string())
                        .unwrap_or_default();
                    ComboBox::from_id_salt(ui.auto_id_with("Relative"))
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| -> PolarsResult<()> {
                            let current_value = &mut self.relative;
                            let fatty_acid = data_frame["FattyAcid"].unique()?;
                            let fatty_acid = fatty_acid
                                .filter(&fatty_acid.fatty_acid().saturated()?)?
                                .sort(Default::default())?
                                .fatty_acid();
                            for index in 0..fatty_acid.len() {
                                if let Some(selected_value) = fatty_acid.get(index)? {
                                    let text = (&selected_value).display(COMMON).to_string();
                                    ui.selectable_value(current_value, Some(selected_value), text);
                                }
                            }
                            Ok(())
                        })
                        .inner
                        .transpose()
                })
                .inner?;
                ui.end_row();

                // DDOF
                // https://numpy.org/devdocs/reference/generated/numpy.std.html
                ui.label("DDOF");
                ui.add(Slider::new(&mut self.ddof, 0..=2));
                ui.end_row();

                // Logarithmic
                ui.label(localize!("logarithmic"));
                ui.checkbox(&mut self.logarithmic, "");
                ui.end_row();

                // Filter
                ui.separator();
                ui.labeled_separator(RichText::new("Filter").heading());
                ui.end_row();

                // Onset temperature filter
                ui.label("Onset").on_hover_text("Onset temperature");
                ui.horizontal(|ui| -> PolarsResult<()> {
                    ComboBox::from_id_salt("OnsetTemperatureFilter")
                        .selected_text(format!("{:?}", self.filter.mode.onset_temperature))
                        .show_ui(ui, |ui| -> PolarsResult<()> {
                            let current_value = &mut self.filter.mode.onset_temperature;
                            for selected_value in
                                &data_frame["Mode"].mode().onset_temperature()?.unique()
                            {
                                ui.selectable_value(
                                    current_value,
                                    selected_value,
                                    AnyValue::from(selected_value).to_string(),
                                );
                            }
                            Ok(())
                        })
                        .inner
                        .transpose()?;
                    if ui.button(TRASH).clicked() {
                        self.filter.mode.onset_temperature = None;
                    }
                    Ok(())
                })
                .inner?;
                ui.end_row();

                // Temperature step filter
                ui.label("Step").on_hover_text("Temperature step");
                ui.horizontal(|ui| -> PolarsResult<()> {
                    ComboBox::from_id_salt("TemperatureStepFilter")
                        .selected_text(format!("{:?}", self.filter.mode.temperature_step))
                        .show_ui(ui, |ui| -> PolarsResult<()> {
                            let current_value = &mut self.filter.mode.temperature_step;
                            for selected_value in &data_frame.mode().temperature_step()?.unique() {
                                ui.selectable_value(
                                    current_value,
                                    selected_value,
                                    AnyValue::from(selected_value).to_string(),
                                );
                            }
                            Ok(())
                        })
                        .inner
                        .transpose()?;
                    if ui.button(TRASH).clicked() {
                        self.filter.mode.temperature_step = None;
                    }
                    Ok(())
                })
                .inner?;
                ui.end_row();

                // Fatty acids filter
                ui.label("Fatty acids");
                // let text = AnyValue::List(Series::from_iter(
                //     self.filter
                //         .fatty_acids
                //         .iter()
                //         .map(|fatty_acid| fatty_acid.to_string()),
                // ))
                // .to_string();
                ui.horizontal(|ui| -> PolarsResult<()> {
                    ComboBox::from_id_salt("FattyAcidsFilter")
                        .selected_text(self.filter.fatty_acids.len().to_string())
                        .show_ui(ui, |ui| -> PolarsResult<()> {
                            let fatty_acid = data_frame["FattyAcid"]
                                .unique()?
                                .sort(Default::default())?
                                .fatty_acid();
                            for index in 0..fatty_acid.len() {
                                if let Ok(Some(fatty_acid)) = fatty_acid.get(index) {
                                    let contains = self.filter.fatty_acids.contains(&fatty_acid);
                                    let mut selected = contains;
                                    ui.toggle_value(
                                        &mut selected,
                                        (&fatty_acid).display(COMMON).to_string(),
                                    );
                                    if selected && !contains {
                                        self.filter.fatty_acids.push(fatty_acid);
                                    } else if !selected && contains {
                                        self.filter.remove(&fatty_acid);
                                    }
                                }
                            }
                            Ok(())
                        })
                        .inner
                        .transpose()?;
                    if ui.button(TRASH).clicked() {
                        self.filter.fatty_acids = Vec::new();
                    }
                    Ok(())
                })
                .inner?;
                ui.end_row();

                // Sort
                ui.separator();
                ui.labeled_separator(RichText::new("Sort").heading());
                ui.end_row();

                ui.label("Sort");
                ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(format!("{:?}", self.sort))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.sort,
                            Sort::FattyAcid,
                            Sort::FattyAcid.text(),
                        )
                        .on_hover_text(Sort::FattyAcid.hover_text());
                        ui.selectable_value(&mut self.sort, Sort::Time, Sort::Time.text())
                            .on_hover_text(Sort::Time.hover_text());
                    });
                ui.end_row();

                // Order
                ui.label("Order");
                ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(self.order.text())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.order,
                            Order::Ascending,
                            Order::Ascending.text(),
                        )
                        .on_hover_text(Order::Ascending.hover_text());
                        ui.selectable_value(
                            &mut self.order,
                            Order::Descending,
                            Order::Descending.text(),
                        )
                        .on_hover_text(Order::Descending.hover_text());
                    })
                    .response
                    .on_hover_text(self.order.hover_text());
                ui.end_row();

                if let Kind::Plot = self.kind {
                    // Plot
                    ui.separator();
                    ui.labeled_separator(RichText::new("Plot").heading());
                    ui.end_row();

                    // Group
                    ui.label("Group");
                    ComboBox::from_id_salt(ui.next_auto_id())
                        .selected_text(self.group.text())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.group,
                                Group::FattyAcid,
                                Group::FattyAcid.text(),
                            )
                            .on_hover_text(Group::FattyAcid.hover_text());
                            ui.selectable_value(
                                &mut self.group,
                                Group::OnsetTemperature,
                                Group::OnsetTemperature.text(),
                            )
                            .on_hover_text(Group::OnsetTemperature.hover_text());
                            ui.selectable_value(
                                &mut self.group,
                                Group::TemperatureStep,
                                Group::TemperatureStep.text(),
                            )
                            .on_hover_text(Group::TemperatureStep.hover_text());
                        })
                        .response
                        .on_hover_text(self.group.hover_text());
                    ui.end_row();

                    // Legend
                    ui.label(localize!("legend"));
                    ui.checkbox(&mut self.legend, "");
                    ui.end_row();
                }
                Ok(())
            })
            .inner
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Group
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Group {
    #[default]
    FattyAcid,
    OnsetTemperature,
    TemperatureStep,
}

impl Text for Group {
    fn text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "Fatty acid",
            Self::OnsetTemperature => "Onset temperature",
            Self::TemperatureStep => "Temperature step",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "Group by fatty acid",
            Self::OnsetTemperature => "Group by onset temperature",
            Self::TemperatureStep => "Group by temperature step",
        }
    }
}

/// Kind
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Kind {
    Plot,
    #[default]
    Table,
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Filter {
    pub(crate) mode: Mode,
    pub(crate) fatty_acids: Vec<FattyAcid>,
}

impl Filter {
    pub const fn new() -> Self {
        Self {
            mode: Mode::new(),
            fatty_acids: Vec::new(),
        }
    }
}

impl Filter {
    fn remove(&mut self, target: &FattyAcid) -> Option<FattyAcid> {
        let position = self
            .fatty_acids
            .iter()
            .position(|source| source == target)?;
        Some(self.fatty_acids.remove(position))
    }
}

/// Mode
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Mode {
    pub(crate) onset_temperature: Option<f64>,
    pub(crate) temperature_step: Option<f64>,
}

impl Mode {
    pub const fn new() -> Self {
        Self {
            onset_temperature: None,
            temperature_step: None,
        }
    }
}

impl Hash for Mode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.map(Float::ord).hash(state);
        self.temperature_step.map(Float::ord).hash(state);
    }
}

/// Sort
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    FattyAcid,
    Time,
}

impl Text for Sort {
    fn text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "Fatty acid",
            Self::Time => "Time",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::FattyAcid => "Sort by atty acid",
            Self::Time => "Sort by Equivalent carbon number and retention time",
        }
    }
}

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(in crate::app) enum Order {
    Ascending,
    Descending,
}

impl Text for Order {
    fn text(&self) -> &'static str {
        match self {
            Self::Ascending => "Ascending",
            Self::Descending => "Descending",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Ascending => "Dscending",
            Self::Descending => "Descending",
        }
    }
}
