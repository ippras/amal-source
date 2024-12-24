pub(crate) use self::{distance::Pane as DistancePane, source::Pane as SourcePane};

use egui::Ui;
use egui_phosphor::regular::{CHART_BAR, TABLE};
use polars::frame::DataFrame;
use serde::{Deserialize, Serialize};

/// Pane
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum Pane {
    Source(SourcePane),
    Distance(DistancePane),
}

impl Pane {
    pub(crate) fn source(data_frame: DataFrame) -> Self {
        Self::Source(SourcePane::new(data_frame))
    }

    pub(crate) fn distance(data_frame: DataFrame) -> Self {
        Self::Distance(DistancePane::new(data_frame))
    }

    pub(crate) const fn icon(&self) -> &str {
        match self {
            Self::Source(_) => CHART_BAR,
            Self::Distance(_) => TABLE,
        }
    }

    pub(crate) const fn title(&self) -> &'static str {
        match self {
            Self::Source(_) => "Source",
            Self::Distance(_) => "Distance",
        }
    }
}

impl Pane {
    fn header(&mut self, ui: &mut Ui) {
        match self {
            Self::Source(pane) => pane.header(ui),
            Self::Distance(pane) => pane.header(ui),
        }
    }

    fn content(&mut self, ui: &mut Ui) {
        match self {
            Self::Source(pane) => pane.content(ui),
            Self::Distance(pane) => pane.content(ui),
        }
    }
}

pub(crate) mod behavior;
pub(crate) mod distance;
pub(crate) mod source;
pub(crate) mod widgets;
