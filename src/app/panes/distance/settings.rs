use crate::{
    app::{MAX_PRECISION, localize},
    special::column::mode::ColumnExt as _,
};
use egui::{ComboBox, Grid, RichText, Slider, Ui, WidgetText, emath::Float};
use egui_ext::LabeledSeparator;
use egui_phosphor::regular::TRASH;
use lipid::fatty_acid::{
    FattyAcid,
    display::{COMMON, DisplayWithOptions},
    polars::ColumnExt,
};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};
use uom::si::{
    f32::Time,
    time::{Units, millisecond, minute, second},
};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) sort: Sort,
    pub(crate) order: Order,

    pub(crate) filter: Filter,
    pub(crate) interpolation: Interpolation,
    pub(crate) filter_onset_temperature: Option<i32>,
    pub(crate) filter_temperature_step: Option<i32>,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            precision: 2,
            resizable: false,
            sticky: 1,
            truncate: false,
            sort: Sort::Ecl,
            order: Order::Descending,

            filter: Filter::new(),
            interpolation: Interpolation::new(),
            filter_onset_temperature: None,
            filter_temperature_step: None,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, data_frame: &DataFrame) {
        Grid::new("calculation").show(ui, |ui| {
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

            // Filter
            ui.separator();
            ui.labeled_separator(RichText::new("Filter").heading());
            ui.end_row();

            // ui.label("Interpolation");
            ui.label(localize!("onset-temperature"));
            ui.add(Slider::new(
                &mut self.interpolation.onset_temperature,
                data_frame["Mode"].mode().onset_temperature_range(),
            ));
            ui.end_row();

            ui.label(localize!("temperature-step"));
            ui.add(Slider::new(
                &mut self.interpolation.temperature_step,
                data_frame["Mode"].mode().temperature_step_range(),
            ));
            ui.end_row();

            ui.label("Filter");
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("FilterFattyAcids")
                    // .selected_text(self.sort.text())
                    .show_ui(ui, |ui| {
                        let fatty_acid = data_frame["FattyAcid"]
                            .unique()
                            .unwrap()
                            .sort(Default::default())
                            .unwrap()
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
                    });
                if ui.button(TRASH).clicked() {
                    self.filter.fatty_acids = Vec::new();
                }
            });
            ui.end_row();

            // Sort
            ui.separator();
            ui.labeled_separator(RichText::new("Sort").heading());
            ui.end_row();

            ui.label("Sort");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(format!("{:?}", self.sort))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort, Sort::Ecl, "ECL");
                    ui.selectable_value(&mut self.sort, Sort::Time, "Time");
                });
            ui.end_row();

            // Order
            ui.label("Order");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(self.order.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.order, Order::Ascending, Order::Ascending.text())
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
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Filter {
    pub(crate) fatty_acids: Vec<FattyAcid>,
}

impl Filter {
    pub const fn new() -> Self {
        Self {
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

/// Interpolation
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Interpolation {
    pub(crate) onset_temperature: f64,
    pub(crate) temperature_step: f64,
}

impl Interpolation {
    pub const fn new() -> Self {
        Self {
            onset_temperature: 0.0,
            temperature_step: 0.0,
        }
    }
}

impl Hash for Interpolation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.ord().hash(state);
        self.temperature_step.ord().hash(state);
    }
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    Time,
    Ecl,
}

/// Mass to charge settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct MassToCharge {
    pub(crate) precision: usize,
}

impl Default for MassToCharge {
    fn default() -> Self {
        Self { precision: 1 }
    }
}

impl MassToCharge {
    pub(crate) fn format(self, value: f32) -> MassToChargeFormat {
        MassToChargeFormat {
            value,
            precision: Some(self.precision),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MassToChargeFormat {
    value: f32,
    precision: Option<usize>,
}

impl MassToChargeFormat {
    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Display for MassToChargeFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let value = self.value;
        if let Some(precision) = self.precision {
            write!(f, "{value:.precision$}")
        } else {
            write!(f, "{value}")
        }
    }
}

impl From<MassToChargeFormat> for WidgetText {
    fn from(value: MassToChargeFormat) -> Self {
        value.to_string().into()
    }
}

/// Retention time settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct RetentionTime {
    pub(crate) precision: usize,
    pub(crate) units: TimeUnits,
}

impl RetentionTime {
    pub(crate) fn format(self, value: f32) -> RetentionTimeFormat {
        RetentionTimeFormat {
            value,
            precision: Some(self.precision),
            units: self.units,
        }
    }
}

impl Default for RetentionTime {
    fn default() -> Self {
        Self {
            precision: 2,
            units: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RetentionTimeFormat {
    value: f32,
    precision: Option<usize>,
    units: TimeUnits,
}

impl RetentionTimeFormat {
    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Display for RetentionTimeFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let time = Time::new::<millisecond>(self.value as _);
        let value = match self.units {
            TimeUnits::Millisecond => time.get::<millisecond>(),
            TimeUnits::Second => time.get::<second>(),
            TimeUnits::Minute => time.get::<minute>(),
        };
        if let Some(precision) = self.precision {
            write!(f, "{value:.precision$}")
        } else {
            write!(f, "{value}")
        }
    }
}

impl From<RetentionTimeFormat> for WidgetText {
    fn from(value: RetentionTimeFormat) -> Self {
        value.to_string().into()
    }
}

/// Time units
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TimeUnits {
    Millisecond,
    #[default]
    Second,
    Minute,
}

impl TimeUnits {
    pub fn abbreviation(&self) -> &'static str {
        Units::from(*self).abbreviation()
    }

    pub fn singular(&self) -> &'static str {
        Units::from(*self).singular()
    }

    pub fn plural(&self) -> &'static str {
        Units::from(*self).plural()
    }
}

impl From<TimeUnits> for Units {
    fn from(value: TimeUnits) -> Self {
        match value {
            TimeUnits::Millisecond => Units::millisecond(millisecond),
            TimeUnits::Second => Units::second(second),
            TimeUnits::Minute => Units::minute(minute),
        }
    }
}

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(in crate::app) enum Order {
    Ascending,
    Descending,
}

impl Order {
    pub(in crate::app) fn text(self) -> &'static str {
        match self {
            Self::Ascending => "Ascending",
            Self::Descending => "Descending",
        }
    }

    pub(in crate::app) fn hover_text(self) -> &'static str {
        match self {
            Self::Ascending => "Dscending",
            Self::Descending => "Descending",
        }
    }
}
