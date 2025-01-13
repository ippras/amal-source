use self::panes::{Pane, behavior::Behavior};
use crate::presets::AGILENT;
use anyhow::Result;
use data::Data;
use eframe::{APP_KEY, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, FontDefinitions, Grid, Id, Label, LayerId, Layout, Order,
    RichText, ScrollArea, TextStyle, TopBottomPanel, menu::bar, warn_if_debug_build,
};
use egui_ext::{DroppedFileExt, HoveredFileExt, LightDarkButton};
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{
        ARROWS_CLOCKWISE, DATABASE, GRID_FOUR, ROCKET, SQUARE_SPLIT_HORIZONTAL,
        SQUARE_SPLIT_VERTICAL, TABS, TRASH,
    },
};
use egui_tiles::{ContainerKind, Tile, Tree};
use egui_tiles_ext::{TreeExt as _, VERTICAL};
use metadata::MetaDataFrame;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, io::Cursor, str, time::Duration};
use tracing::{error, info, trace};

macro localize($text:literal) {
    $text
}

/// IEEE 754-2008
const MAX_PRECISION: usize = 16;
const MAX_TEMPERATURE: f64 = 250.0;
const _NOTIFICATIONS_DURATION: Duration = Duration::from_secs(15);
const SIZE: f32 = 32.0;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    data: Data,
    reactive: bool,
    // Panels
    left_panel: bool,
    // Panes
    tree: Tree<Pane>,
    behavior: Behavior,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: Data::default(),
            reactive: true,
            left_panel: true,
            tree: Tree::empty("tree"),
            behavior: Default::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = FontDefinitions::default();
        add_to_fonts(&mut fonts, Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        // return Default::default();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        cc.storage
            .and_then(|storage| get_value(storage, APP_KEY))
            .unwrap_or_default()
    }

    fn drag_and_drop(&mut self, ctx: &egui::Context) {
        // Preview hovering files
        if let Some(text) = ctx.input(|input| {
            (!input.raw.hovered_files.is_empty()).then(|| {
                let mut text = String::from("Dropping files:");
                for file in &input.raw.hovered_files {
                    write!(text, "\n{}", file.display()).ok();
                }
                text
            })
        }) {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
        // Parse dropped files
        if let Some(dropped_files) = ctx.input(|input| {
            (!input.raw.dropped_files.is_empty()).then_some(input.raw.dropped_files.clone())
        }) {
            info!(?dropped_files);
            for dropped_file in dropped_files {
                if let Err(error) = || -> Result<()> {
                    let frame = MetaDataFrame::read(Cursor::new(dropped_file.bytes()?))?;
                    trace!(?frame);
                    self.data.stack(&frame.data)?;
                    ctx.request_repaint();
                    Ok(())
                }() {
                    error!(%error);
                }
                // match ron(&dropped_file) {
                //     Ok(data_frame) => {
                //         trace!(?data_frame);
                //         self.data.stack(&data_frame).unwrap();
                //         if !self.tree.tiles.is_empty() {
                //             self.tree = Tree::empty("tree");
                //         }
                //         // self.tree
                //         //     .insert_pane(Pane::source(self.data.data_frame.clone()));
                //         // self.tree
                //         //     .insert_pane(Pane::distance(self.data.data_frame.clone()));
                //         trace!(?self.data);
                //     }
                //     Err(error) => {
                //         error!(%error);
                //         // self.toasts
                //         //     .error(format!("{}: {error}", dropped.display()))
                //         //     .set_closable(true)
                //         //     .set_duration(Some(NOTIFICATIONS_DURATION));
                //         continue;
                //     }
                // };
            }
            // TODO
            // println!("data_frame: {}", self.data.data_frame);
            // let data_frame = self
            //     .data
            //     .data_frame
            //     .clone()
            //     .lazy()
            //     .select([
            //         as_struct(vec![
            //             col("OnsetTemperature").alias("OnsetTemperature"),
            //             col("TemperatureStep").alias("TemperatureStep"),
            //         ])
            //         .alias("Mode"),
            //         col("FA"),
            //         col("Time"),
            //     ])
            //     .cache()
            //     .sort(["Mode"], SortMultipleOptions::new())
            //     .select([all()
            //         .sort_by(&[col("Time").list().mean()], SortMultipleOptions::new())
            //         .over([col("Mode")])])
            //     .collect()
            //     .unwrap();
            // println!("data_frame: {data_frame}");
            // self.tree
            //     .insert_pane::<VERTICAL>(Pane::source(data_frame.clone()));
            // self.tree
            //     .insert_pane::<VERTICAL>(Pane::distance(data_frame.clone()));
            // data::save("data_frame.bin", data::Format::Bin, data_frame).unwrap();
        }
    }
}

impl App {
    fn panels(&mut self, ctx: &egui::Context) {
        self.top_panel(ctx);
        self.bottom_panel(ctx);
        self.left_panel(ctx);
        self.central_panel(ctx);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                warn_if_debug_build(ui);
                ui.label(RichText::new(env!("CARGO_PKG_VERSION")).small());
                ui.separator();
            });
        });
    }

    // Central panel
    fn central_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            self.tree.ui(&mut self.behavior, ui);
            if let Some(id) = self.behavior.close.take() {
                self.tree.tiles.remove(id);
            }
        });
    }

    // Left panel
    fn left_panel(&mut self, ctx: &egui::Context) {
        // SidePanel::left("left_panel")
        //     .frame(egui::Frame::side_top_panel(&ctx.style()))
        //     .resizable(true)
        //     .show_animated(ctx, self.left_panel, |ui| {
        //         ScrollArea::vertical().show(ui, |ui| {});
        //     });
    }

    // Top panel
    fn top_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.light_dark_button(SIZE);
                    ui.separator();
                    ui.toggle_value(&mut self.reactive, RichText::new(ROCKET).size(SIZE))
                        .on_hover_text("reactive")
                        .on_hover_text(localize!("reactive_description_enabled"))
                        .on_disabled_hover_text(localize!("reactive_description_disabled"));
                    ui.separator();
                    // Reset app
                    if ui
                        .button(RichText::new(TRASH).size(SIZE))
                        .on_hover_text(localize!("reset_application"))
                        .clicked()
                    {
                        *self = Self {
                            reactive: self.reactive,
                            ..Default::default()
                        };
                    }
                    ui.separator();
                    // Reset GUI
                    if ui
                        .button(RichText::new(ARROWS_CLOCKWISE).size(SIZE))
                        .on_hover_text(localize!("reset_gui"))
                        .clicked()
                    {
                        ui.memory_mut(|memory| {
                            memory.data = Default::default();
                        });
                    }
                    ui.separator();
                    if ui
                        .button(RichText::new(SQUARE_SPLIT_VERTICAL).size(SIZE))
                        .on_hover_text(localize!("vertical"))
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Vertical);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(SQUARE_SPLIT_HORIZONTAL).size(SIZE))
                        .on_hover_text(localize!("horizontal"))
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Horizontal);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(GRID_FOUR).size(SIZE))
                        .on_hover_text(localize!("grid"))
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Grid);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(TABS).size(SIZE))
                        .on_hover_text(localize!("tabs"))
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Tabs);
                            }
                        }
                    }
                    ui.separator();
                    ui.menu_button(RichText::new(DATABASE).size(SIZE), |ui| {
                        let mut response = ui
                            .button(RichText::new(format!("{DATABASE} IPPRAS/Agilent")).heading());
                        response = response.on_hover_ui(|ui| {
                            let meta = &AGILENT.meta;
                            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                ui.label("Name");
                                ui.label(&meta.name);
                                ui.end_row();

                                if !meta.description.is_empty() {
                                    ui.label("Description");
                                    ui.add(Label::new(&meta.description).truncate());
                                    ui.end_row();
                                }

                                ui.label("Authors");
                                ui.label(meta.authors.join(", "));
                                ui.end_row();

                                if let Some(version) = &meta.version {
                                    ui.label("Version");
                                    ui.label(version.to_string());
                                    ui.end_row();
                                }

                                if let Some(date) = &meta.date {
                                    ui.label("Date");
                                    ui.label(date.to_string());
                                    ui.end_row();
                                }
                            });
                        });
                        if response.clicked() {
                            self.tree
                                .insert_pane::<VERTICAL>(Pane::source(AGILENT.clone()));
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                });
            });
        });
    }
}

impl App {
    fn distance(&mut self, ctx: &egui::Context) {
        if let Some(frame) = ctx.data_mut(|data| data.remove_temp(Id::new("Distance"))) {
            self.tree.insert_pane::<VERTICAL>(Pane::distance(frame));
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.distance(ctx);
        self.panels(ctx);
        self.drag_and_drop(ctx);
        if self.reactive {
            ctx.request_repaint();
        }
    }
}

mod computers;
mod data;
mod panes;
mod text;
