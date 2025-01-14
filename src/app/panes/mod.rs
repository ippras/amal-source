pub(crate) use self::{distance::Pane as DistancePane, source::Pane as SourcePane};

use egui::{Response, Ui, Vec2, vec2};
use metadata::MetaDataFrame;
use serde::{Deserialize, Serialize};

const MARGIN: Vec2 = vec2(4.0, 2.0);

/// Pane
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum Pane {
    Source(SourcePane),
    Distance(DistancePane),
}

impl Pane {
    pub(crate) fn source(frame: MetaDataFrame) -> Self {
        Self::Source(SourcePane::new(frame))
    }

    pub(crate) fn distance(frame: MetaDataFrame) -> Self {
        Self::Distance(DistancePane::new(frame))
    }

    pub(crate) const fn title(&self) -> &'static str {
        match self {
            Self::Source(_) => "Source",
            Self::Distance(_) => "Distance",
        }
    }
}

impl Pane {
    fn header(&mut self, ui: &mut Ui) -> Response {
        match self {
            Self::Source(pane) => pane.header(ui),
            Self::Distance(pane) => pane.header(ui),
        }
    }

    fn body(&mut self, ui: &mut Ui) {
        match self {
            Self::Source(pane) => pane.body(ui),
            Self::Distance(pane) => pane.body(ui),
        }
    }
}

pub(crate) mod behavior;
pub(crate) mod distance;
pub(crate) mod source;
pub(crate) mod widgets;
