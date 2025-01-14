use self::{control::Control, settings::Settings};
use crate::{
    app::{
        computers::{DistanceComputed, DistanceKey},
        localize,
    },
    utils::save,
};
use egui::{CursorIcon, Response, RichText, Ui, Window, util::hash};
use egui_phosphor::regular::{ARROWS_HORIZONTAL, EXCLUDE, FLOPPY_DISK, GEAR};
use metadata::MetaDataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use table::TableView;
use tracing::error;

/// Distance pane
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
        let mut response = ui.heading(EXCLUDE).on_hover_text(localize!("distance"));
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
        // Save
        let name = format!("{}.distance.ipc", self.source.meta.title());
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
            memory.caches.cache::<DistanceComputed>().get(DistanceKey {
                data_frame: &self.source.data,
                settings: &self.control.settings,
            })
        });
        TableView::new(&self.target, &self.control.settings).show(ui);
    }

    fn window(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Distance settings"))
            .id(ui.auto_id_with("DistanceSettings"))
            .open(&mut self.control.open)
            .show(ui.ctx(), |ui| {
                self.control.settings.ui(ui, &self.source.data);
            });
    }
}

pub(crate) mod settings;

mod control;
mod table;
