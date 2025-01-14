use self::{
    control::Control,
    plot::PlotView,
    settings::{Kind, Settings},
    table::TableView,
};
use crate::{
    app::{
        computers::{SourceComputed, SourceKey},
        localize,
    },
    utils::save,
};
use egui::{CursorIcon, Id, Response, RichText, Ui, Window, util::hash};
use egui_phosphor::regular::{ARROWS_HORIZONTAL, CHART_BAR, EXCLUDE, FLOPPY_DISK, GEAR, TABLE};
use metadata::MetaDataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;

/// Source pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: MetaDataFrame,
    pub(crate) target: DataFrame,
    pub(crate) control: Control,
}

impl Pane {
    pub(crate) const fn new(frame: MetaDataFrame) -> Self {
        Self {
            source: frame,
            target: DataFrame::empty(),
            control: Control::new(),
        }
    }

    pub(super) fn header(&mut self, ui: &mut Ui) -> Response {
        ui.visuals_mut().button_frame = false;
        let mut response = ui.heading(TABLE).on_hover_text(localize!("source"));
        response |= ui.heading(self.source.meta.title());
        response = response
            .on_hover_text(format!("{:x}", hash(&self.source)))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // Resize
        ui.toggle_value(
            &mut self.control.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(localize!("resize"));
        ui.separator();
        // Settings
        ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading());
        ui.separator();
        // Kind
        match self.control.settings.kind {
            Kind::Plot => {
                if ui.button(RichText::new(TABLE).heading()).clicked() {
                    self.control.settings.kind = Kind::Table;
                }
            }
            Kind::Table => {
                if ui.button(RichText::new(CHART_BAR).heading()).clicked() {
                    self.control.settings.kind = Kind::Plot;
                }
            }
        };
        ui.separator();
        // Distance
        if ui.button(RichText::new(EXCLUDE).heading()).clicked() {
            ui.data_mut(|data| {
                data.insert_temp(
                    Id::new("Distance"),
                    MetaDataFrame::new(self.source.meta.clone(), self.target.clone()),
                )
            })
        }
        ui.separator();
        // Save
        let name = format!("{}.source.ipc", self.source.meta.title());
        if ui
            .button(RichText::new(FLOPPY_DISK).heading())
            .on_hover_text(&name)
            .clicked()
        {
            if let Err(error) = save(&name, Some(&self.source.meta), &mut self.target) {
                error!(%error);
            }
        }
        ui.separator();
        response
    }

    pub(super) fn body(&mut self, ui: &mut Ui) {
        self.window(ui);
        self.target = ui.memory_mut(|memory| {
            memory.caches.cache::<SourceComputed>().get(SourceKey {
                data_frame: &self.source.data,
                settings: &self.control.settings,
            })
        });
        match self.control.settings.kind {
            Kind::Plot => PlotView::new(&self.target, &self.control.settings).show(ui),
            Kind::Table => TableView::new(&self.target, &self.control.settings).show(ui),
        };
    }

    fn window(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Source settings"))
            .id(ui.auto_id_with("SourceSettings"))
            .open(&mut self.control.open)
            .show(ui.ctx(), |ui| {
                if let Err(error) = self.control.settings.ui(ui, &self.source.data) {
                    error!(%error);
                }
            });
    }
}

pub(crate) mod settings;

mod control;
mod plot;
mod table;
