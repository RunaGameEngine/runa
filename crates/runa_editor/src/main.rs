#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::ViewportBuilder;
use eframe::{App, egui};
use egui::containers::menu;
use egui::{CentralPanel, IconData, RichText, SidePanel, TopBottomPanel, UiBuilder};
use runa_core::World;

mod preview;

struct Editor {
    world: World,
    show_content_browser: bool,
    selected_object_id: Option<usize>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            world: World::default(),
            selected_object_id: None,
            show_content_browser: false,
        }
    }
}

fn load_icon() -> Option<IconData> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes).ok()?;
    let image = image.to_rgba8();
    let (width, height) = image.dimensions();

    Some(IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

impl App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New World").clicked() { /* ... */ }
                    if ui.button("Open...").clicked() { /* ... */ }
                    if ui.button("Save").clicked() { /* ... */ }
                    if ui.button("Save As...").clicked() { /* ... */ }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() { /* ... */ }
                    if ui.button("Redo").clicked() { /* ... */ }
                    ui.separator();
                    if ui.button("Project Settings").clicked() { /* ... */ }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Content Browser (Ctrl+Space)").clicked() {
                        self.show_content_browser = !self.show_content_browser;
                    }
                });

                // Window, Help и т.д. — по желанию
            });
        });

        // ctx.

        SidePanel::left("hierarchy_panel")
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Hierarchy");
                ui.separator();
                ui.label("Empty World");
            });

        SidePanel::right("inspector_panel")
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();
                ui.label("No object selected");
            });

        CentralPanel::default().show(ctx, |ui| {
            // === Кнопка Play поверх viewport'а ===
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let available = ui.available_rect_before_wrap();
                // Позиционируем кнопку в левом верхнем углу viewport'а
                ui.scope_builder(
                    UiBuilder::new().layer_id(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("play_button"),
                    )),
                    |ui| {
                        if ui
                            .button(RichText::new("▶ Play").color(egui::Color32::WHITE))
                            .clicked()
                        {
                            let world_ref = &self.world; // borrow
                            std::thread::spawn(move || {
                                // Но! Нельзя передать &World в другой поток — не Send!
                            });
                        }
                    },
                );
            });

            // === Тут будет preview или placeholder ===
            ui.centered_and_justified(|ui| {
                ui.label("Viewport (Preview will appear here later)");
            });
        });

        if ctx.input_mut(|ui| ui.consume_key(egui::Modifiers::CTRL, egui::Key::Space)) {
            self.show_content_browser = !self.show_content_browser;
        }

        if self.show_content_browser {
            // Полупрозрачный затемняющий фон поверх всего
            let area = egui::Area::new(egui::Id::new("content_browser_overlay"))
                .order(egui::Order::Background) // под основным UI
                .fixed_pos(egui::Pos2::ZERO)
                .interactable(false);

            // area.show(ctx, |ui| {
            //     ui.scope_builder(egui::UiBuilder::new().max_rect(ctx.content_rect()), |ui| {
            //         ui.painter().rect_filled(
            //             ctx.content_rect(),
            //             0.0,
            //             egui::Color32::from_black_alpha(128), // полупрозрачный чёрный
            //         );
            //     });
            // });

            // Сам Content Browser — растянутый на всю ширину и высоту снизу
            egui::Area::new(egui::Id::new("content_browser"))
                .anchor(egui::Align2::LEFT_BOTTOM, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .interactable(true)
                .show(ctx, |ui| {
                    // Стиль: тёмный, как UE
                    ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                    ui.visuals_mut().panel_fill = egui::Color32::from_rgb(20, 20, 20);
                    ui.visuals_mut().window_fill = egui::Color32::from_rgb(20, 20, 20);
                    ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 75);
                    ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::from_rgb(80, 80, 100);

                    // Установка размеров: на всю ширину и фиксированную высоту от низа
                    let content_rect = ctx.content_rect();
                    let available_height = 400.0; // Высота 400px от низа экрана

                    // Создаём прямоугольник для контент-браузера
                    let browser_rect = egui::Rect::from_min_size(
                        egui::pos2(0.0, content_rect.max.y - available_height),
                        egui::Vec2::new(content_rect.width(), available_height),
                    );

                    ui.scope_builder(egui::UiBuilder::new().max_rect(browser_rect), |ui| {
                        egui::Frame::window(&ui.style())
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.set_min_height(ui.available_height());

                                ui.heading("Content Browser");
                                ui.separator();

                                // Поиск
                                // ui.text_edit_singleline(&mut self.search_query);

                                // Список скриптов
                                // egui::ScrollArea::vertical().show(ui, |ui| {
                                //     for script_name in &self.registered_scripts {
                                //         if ui.button(script_name).clicked() {
                                //             // TODO: добавить объект в мир
                                //             self.show_content_browser = false;
                                //         }
                                //     }
                                // });
                            });
                    });

                    // Закрытие по клику вне области (опционально)
                    // Это уже обрабатывается egui автоматически, если кликнуть на overlay
                });
        }
    }
}

fn main() -> eframe::Result {
    let icon_data = load_icon();

    let viewport = ViewportBuilder::default()
        .with_title("Runa Editor")
        .with_inner_size([1280.0, 720.0])
        .with_fullscreen(false);

    let viewport = if let Some(icon) = icon_data {
        viewport.with_icon(icon)
    } else {
        viewport
    };

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Runa Editor",
        options,
        Box::new(|cc| {
            setup_style(&cc.egui_ctx);
            Ok(Box::new(Editor::default()))
        }),
    )
}

fn setup_style(ctx: &egui::Context) {
    use egui::{Color32, Style, Visuals};

    let mut style = Style::default();

    // Тёмная тема в духе UE5 / Zed
    style.visuals = Visuals::dark();
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 40, 45);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 70);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(80, 80, 90);
    style.visuals.panel_fill = Color32::from_rgb(10, 10, 10);
    style.visuals.window_fill = Color32::from_rgb(25, 25, 30);
    style.visuals.code_bg_color = Color32::from_rgb(255, 255, 255);
    style.visuals.hyperlink_color = Color32::from_rgb(100, 180, 255);
    style.visuals.selection.bg_fill = Color32::from_rgb(60, 100, 160);

    // Опционально: настроить шрифт
    style
        .text_styles
        .get_mut(&egui::TextStyle::Body)
        .unwrap()
        .size = 18.0;

    ctx.set_style(style);
}
