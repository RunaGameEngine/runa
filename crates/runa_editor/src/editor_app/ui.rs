use super::*;

impl<'window> EditorApp<'window> {
    pub(super) fn draw_ui(&mut self) -> egui::FullOutput {
        let window = self.window.as_ref().unwrap();
        let egui_state = self.egui_state.as_mut().unwrap();
        let raw_input = egui_state.take_egui_input(window);
        let egui_ctx = self.egui_ctx.clone();
        egui_ctx.run_ui(raw_input, |ctx| {
            self.build_ui(ctx);
        })
    }

    fn build_ui(&mut self, ctx: &egui::Context) {
        let ui_scale = ctx.zoom_factor().max(0.75);
        egui::Panel::top("editor_top_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project...").clicked() {
                        self.project_dialog.open = true;
                        ui.close();
                    }
                    if ui.button("Open Project...").clicked() {
                        self.open_project_dialog();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("New World").clicked() {
                        self.new_world();
                        ui.close();
                    }
                    if ui.button("Open World...").clicked() {
                        self.open_world_dialog();
                        ui.close();
                    }
                    if ui.button("Save World").clicked() {
                        self.save_current_world();
                        ui.close();
                    }
                    if ui.button("Save World As...").clicked() {
                        self.save_world_as_dialog();
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Editor Settings").clicked() {
                        self.editor_settings_open = true;
                        ui.close();
                    }
                    if ui.button("Project Settings").clicked() {
                        self.project_settings_open = true;
                        ui.close();
                    }
                });
                ui.menu_button("Build", |ui| {
                    if ui.button("Build Settings").clicked() {
                        self.build_settings_open = true;
                        ui.close();
                    }
                    let build_enabled =
                        self.project_session.is_some() && self.build_process.is_none();
                    let build_label = if self.build_process.is_some() {
                        "Building..."
                    } else {
                        "Build Game"
                    };
                    if ui
                        .add_enabled(build_enabled, egui::Button::new(build_label))
                        .clicked()
                    {
                        self.build_project();
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.panels.hierarchy, "Hierarchy");
                    ui.checkbox(&mut self.panels.inspector, "Inspector");
                    ui.checkbox(&mut self.panels.bottom_bar, "Bottom Bar");
                });
                ui.separator();

                let play_label = if self.runtime_process.is_some() {
                    "Stop"
                } else {
                    "Play In Window"
                };
                let play_response = ui.add_enabled(
                    self.project_session.is_some(),
                    egui::Button::new(play_label),
                );
                if play_response.clicked() {
                    if self.runtime_process.is_some() {
                        self.stop_project();
                    } else {
                        self.play_project();
                    }
                }

                let refresh_label = if self.place_object.refresh_in_progress {
                    "Refreshing..."
                } else {
                    "Refresh Project Metadata"
                };
                let refresh_response = ui.add_enabled(
                    self.project_session.is_some() && !self.place_object.refresh_in_progress,
                    egui::Button::new(refresh_label),
                );
                if refresh_response.clicked() {
                    self.refresh_project_metadata(true);
                }

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(self.window_title()).strong());
                });
            });
        });

        egui::Panel::bottom("status_bar")
            .resizable(false)
            .exact_size(24.0 * ui_scale)
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.label(&self.status_line);
            });

        if self.panels.bottom_bar {
            let max_bottom_bar_height = (ctx.content_rect().height() - 140.0).max(120.0);
            self.bottom_bar_height = self.bottom_bar_height.clamp(80.0, max_bottom_bar_height);
            let content_rect = ctx.content_rect();
            let side_margin = 12.0;
            let bottom_gap = 32.0 * ui_scale;
            let overlay_width = (content_rect.width() - side_margin * 2.0).max(240.0);
            let overlay_pos = egui::pos2(
                content_rect.left() + side_margin,
                content_rect.bottom() - bottom_gap - self.bottom_bar_height,
            );
            egui::Area::new("bottom_bar_overlay".into())
                .order(egui::Order::Middle)
                .fixed_pos(overlay_pos)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(ui.visuals().panel_fill)
                        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                        .corner_radius(egui::CornerRadius::same(style::spacing::CORNER_RADIUS))
                        .show(ui, |ui| {
                            ui.set_min_size(egui::vec2(overlay_width, self.bottom_bar_height));
                            ui.set_max_size(egui::vec2(overlay_width, self.bottom_bar_height));

                            let handle_height = 10.0;
                            let (handle_rect, handle_response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), handle_height),
                                egui::Sense::click_and_drag(),
                            );
                            if handle_response.hovered() || handle_response.dragged() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                            }
                            if handle_response.dragged() {
                                self.bottom_bar_height = (self.bottom_bar_height
                                    - handle_response.drag_delta().y)
                                    .clamp(80.0, max_bottom_bar_height);
                            }
                            let handle_fill = if handle_response.dragged() {
                                ui.visuals().widgets.active.bg_fill
                            } else if handle_response.hovered() {
                                ui.visuals().widgets.hovered.bg_fill
                            } else {
                                ui.visuals().widgets.inactive.bg_fill
                            };
                            ui.painter().rect_filled(
                                handle_rect,
                                4.0,
                                handle_fill.gamma_multiply(0.35),
                            );
                            ui.painter().rect_filled(
                                handle_rect.shrink2(egui::vec2(48.0, 3.0)),
                                4.0,
                                handle_fill,
                            );
                            ui.horizontal(|ui| {
                                let browser_selected = self.bottom_tab == BottomTab::ContentBrowser;
                                if ui
                                    .selectable_label(browser_selected, "Content Browser")
                                    .clicked()
                                {
                                    self.bottom_tab = BottomTab::ContentBrowser;
                                }
                                let console_selected = self.bottom_tab == BottomTab::Console;
                                if ui.selectable_label(console_selected, "Console").clicked() {
                                    self.bottom_tab = BottomTab::Console;
                                }
                                ui.separator();
                                match self.bottom_tab {
                                    BottomTab::ContentBrowser => {
                                        if ui.button("Import").clicked() {
                                            self.content_browser
                                                .import_into_current_dir(&self.settings);
                                            if let Some(message) =
                                                self.content_browser.take_message()
                                            {
                                                self.status_line = message;
                                            }
                                        }
                                        ui.separator();
                                        ui.label(self.content_browser.current_dir_display());
                                    }
                                    BottomTab::Console => {
                                        if ui.button("Clear").clicked() {
                                            self.output_lines.clear();
                                        }
                                    }
                                }
                            });
                            ui.separator();
                            let body_height = (ui.available_height() - 8.0).max(0.0);
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), body_height),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| match self.bottom_tab {
                                    BottomTab::ContentBrowser => {
                                        self.content_browser.ui(ui, &self.settings);
                                        if let Some(message) = self.content_browser.take_message() {
                                            self.status_line = message;
                                        }
                                    }
                                    BottomTab::Console => {
                                        egui::ScrollArea::vertical()
                                            .auto_shrink([false, false])
                                            .stick_to_bottom(true)
                                            .id_salt("editor_output_scroll")
                                            .show(ui, |ui| {
                                                ui.set_min_width(ui.available_width());
                                                for line in &self.output_lines {
                                                    ui.monospace(line);
                                                }
                                            });
                                    }
                                },
                            );
                        });
                });
        }

        if self.panels.hierarchy {
            egui::Panel::left("hierarchy_panel")
                .resizable(true)
                .default_size(240.0)
                .min_size(180.0)
                .show(ctx, |ui| {
                    ui.heading("Hierarchy");
                    ui.separator();
                    let hierarchy_items: Vec<(ObjectId, String)> = self
                        .world_object_ids()
                        .into_iter()
                        .filter_map(|object_id| {
                            self.world
                                .get(object_id)
                                .map(|object| (object_id, helpers::object_title(object)))
                        })
                        .collect();
                    egui::ScrollArea::vertical()
                        .id_salt("hierarchy_scroll")
                        .show(ui, |ui| {
                            for (object_id, title) in &hierarchy_items {
                                let selected = self.selection == Some(*object_id);
                                let response = ui.selectable_label(selected, title);
                                if response.clicked() {
                                    self.selection = Some(*object_id);
                                }
                                response.context_menu(|ui| {
                                    self.selection = Some(*object_id);
                                    self.hierarchy_context_menu_ui(ui, Some(*object_id));
                                });
                            }
                            let blank_response = ui.allocate_response(
                                egui::vec2(ui.available_width(), ui.available_height().max(24.0)),
                                egui::Sense::click(),
                            );
                            blank_response.context_menu(|ui| {
                                self.hierarchy_context_menu_ui(ui, None);
                            });
                        });
                });
        }

        if self.panels.inspector {
            egui::Panel::right("inspector_panel")
                .resizable(true)
                .default_size(320.0)
                .min_size(220.0)
                .show(ctx, |ui| {
                    ui.heading("Inspector");
                    ui.separator();
                    if let Some(object_id) = self.selection {
                        if let Some(object) = self.world.get_mut(object_id) {
                            let project_root = self
                                .project_session
                                .as_ref()
                                .map(|session| session.project.root_dir.as_path());
                            let inspector_actions = inspector_ui(ui, object, project_root);
                            for removal in inspector_actions.removals {
                                match removal.target {
                                    crate::inspector::InspectorRemovalTarget::RuntimeType {
                                        type_id,
                                        type_name,
                                    } => {
                                        if object.remove_component_type_id(type_id) {
                                            self.status_line = format!("Removed {}.", type_name);
                                        }
                                    }
                                    crate::inspector::InspectorRemovalTarget::SerializedType {
                                        kind,
                                        type_name,
                                    } => {
                                        if let Some(storage) =
                                            object.get_component_mut::<SerializedTypeStorage>()
                                        {
                                            if storage.remove(kind, &type_name) {
                                                self.status_line = format!("Removed {}.", type_name);
                                            }
                                        }
                                    }
                                }
                            }
                            ui.separator();
                            self.inspector_actions_ui(ui, object_id);
                        } else {
                            ui.label("Selection is out of bounds.");
                        }
                    } else {
                        ui.label("Select an object in the hierarchy.");
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("World");
                if let Some(path) = self.current_world_path() {
                    ui.label(path.display().to_string());
                } else {
                    ui.label("Unsaved world");
                }

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Gizmo").clicked() {
                        self.gizmo_settings_open = !self.gizmo_settings_open;
                    }

                    let camera_icon = crate::editor_textures::load_component_icon(
                        ui.ctx(),
                        "viewport_toolbar_camera_icon",
                        "c-Camera",
                    );
                    let camera_button = ui.add(
                        egui::Button::image(
                            egui::Image::new(&camera_icon)
                                .fit_to_exact_size(egui::vec2(18.0, 18.0)),
                        )
                        .frame(true),
                    );
                    if camera_button.clicked() {
                        self.view_settings_open = !self.view_settings_open;
                    }

                    if ui.button("Rendering").clicked() {
                        self.rendering_settings_open = !self.rendering_settings_open;
                    }
                    if ui.button("Frame Selected").clicked() {
                        self.frame_selected_object();
                    }
                });
            });
            ui.separator();

            let desired_size = ui.available_size().max(egui::vec2(64.0, 64.0));
            let pixels_per_point = ctx.pixels_per_point();
            self.pending_viewport_size = (
                (desired_size.x * pixels_per_point).round().max(1.0) as u32,
                (desired_size.y * pixels_per_point).round().max(1.0) as u32,
            );

            let frame = egui::Frame::canvas(ui.style())
                .fill(style::VIEWPORT_BACKGROUND)
                .inner_margin(egui::Margin::same(0));

            frame.show(ui, |ui| {
                if let Some(texture_id) = self.viewport_texture_id {
                    let response = ui.add(
                        egui::Image::new((texture_id, desired_size))
                            .sense(egui::Sense::click_and_drag()),
                    );
                    self.viewport_hovered = response.hovered();
                    self.handle_viewport_interaction(ui, ctx, &response);
                } else {
                    self.viewport_hovered = false;
                    ui.allocate_ui_with_layout(
                        desired_size,
                        Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.label("Viewport is initializing...");
                        },
                    );
                }
            });
        });

        self.viewport_settings_window(ctx);
        self.rendering_settings_window(ctx);
        self.gizmo_settings_window(ctx);
        self.editor_settings_window(ctx);
        self.project_settings_window(ctx);
        self.build_settings_window(ctx);
        self.project_dialog_window(ctx);
        self.project_loading_overlay(ctx);
    }

    pub(super) fn render_frame(&mut self) {
        let full_output = self.draw_ui();
        self.ensure_viewport_target();
        self.update_scene_preview();

        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(egui_state) = self.egui_state.as_mut() else {
            return;
        };
        egui_state.handle_platform_output(window, full_output.platform_output);

        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let Some(egui_renderer) = self.egui_renderer.as_mut() else {
            return;
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(renderer.device(), renderer.queue(), *id, image_delta);
        }

        let size = window.inner_size();
        let pixels_per_point = window.scale_factor() as f32;
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width.max(1), size.height.max(1)],
            pixels_per_point,
        };

        let surface_texture = match renderer.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            _ => return,
        };
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            renderer
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Editor UI Encoder"),
                });

        let mut command_buffers = egui_renderer.update_buffers(
            renderer.device(),
            renderer.queue(),
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor UI Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(style::RENDER_CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            let mut render_pass = render_pass.forget_lifetime();
            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        command_buffers.push(encoder.finish());
        renderer.queue().submit(command_buffers);
        surface_texture.present();

        for id in &full_output.textures_delta.free {
            egui_renderer.free_texture(id);
        }
    }

    fn project_loading_overlay(&mut self, ctx: &egui::Context) {
        if self.project_load.is_none() {
            return;
        }

        egui::Window::new("Loading Project")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("Project is loading in the background.");
                ui.add(egui::Spinner::new());
                ui.label("Preparing bridge, loading world, and caching placeable objects.");
            });
    }

    fn hierarchy_context_menu_ui(&mut self, ui: &mut egui::Ui, target_id: Option<ObjectId>) {
        if ui.button("Create Empty Object").clicked() {
            self.create_empty_object();
            ui.close();
            return;
        }
        self.create_from_archetype_menu_ui(ui);
        ui.separator();

        if let Some(object_id) = target_id {
            if ui.button("Copy").clicked() {
                self.copy_object(object_id, false);
                ui.close();
            }
            if ui.button("Cut").clicked() {
                self.copy_object(object_id, true);
                ui.close();
            }
            if ui
                .add_enabled(
                    self.hierarchy_clipboard.is_some(),
                    egui::Button::new("Paste"),
                )
                .clicked()
            {
                self.paste_object(target_id);
                ui.close();
            }
            if ui.button("Delete").clicked() {
                self.delete_object(object_id);
                ui.close();
            }
            ui.separator();
        } else if ui
            .add_enabled(
                self.hierarchy_clipboard.is_some(),
                egui::Button::new("Paste"),
            )
            .clicked()
        {
            self.paste_object(None);
            ui.close();
            return;
        }
    }

    fn inspector_actions_ui(&mut self, ui: &mut egui::Ui, object_id: ObjectId) {
        ui.horizontal_wrapped(|ui| {
            self.add_registered_type_menu_ui(ui, object_id, RegisteredTypeKind::Component);
            self.add_registered_type_menu_ui(ui, object_id, RegisteredTypeKind::Script);
        });
    }

    fn add_registered_type_menu_ui(
        &mut self,
        ui: &mut egui::Ui,
        object_id: ObjectId,
        kind: RegisteredTypeKind,
    ) {
        let label = match kind {
            RegisteredTypeKind::Component => "Add Component",
            RegisteredTypeKind::Script => "Add Script",
        };

        ui.menu_button(label, |ui| {
            let Some(object) = self.world.get(object_id) else {
                ui.label("Object not found.");
                return;
            };

            let mut registered_types: Vec<(Option<TypeId>, String, bool, Option<ProjectRegisteredTypeRecord>)> = self
                .runtime_registry()
                .types()
                .registered_types()
                .into_iter()
                .filter(|metadata| metadata.kind() == kind)
                .filter(|metadata| metadata.type_id() != TypeId::of::<Tilemap>())
                .filter(|metadata| !object.has_component_type_id(metadata.type_id()))
                .map(|metadata| {
                    (
                        Some(metadata.type_id()),
                        helpers::short_type_name(metadata.type_name()).to_string(),
                        self.runtime_registry()
                            .types()
                            .has_object_factory(metadata.type_id()),
                        None,
                    )
                })
                .collect();

            let project_kind = match kind {
                RegisteredTypeKind::Component => ProjectRegisteredTypeKind::Component,
                RegisteredTypeKind::Script => ProjectRegisteredTypeKind::Script,
            };
            for metadata in self
                .place_object
                .registered_types
                .iter()
                .filter(|metadata| metadata.kind == project_kind)
                .filter(|metadata| metadata.source == runa_project::ProjectRegistrationSource::User)
            {
                let already_attached_as_stub = object
                    .get_component::<SerializedTypeStorage>()
                    .map(|storage| {
                        let kind = match metadata.kind {
                            ProjectRegisteredTypeKind::Component => SerializedTypeKind::Component,
                            ProjectRegisteredTypeKind::Script => SerializedTypeKind::Script,
                        };
                        storage
                            .entries
                            .iter()
                            .any(|entry| entry.kind == kind && entry.type_name == metadata.type_name)
                    })
                    .unwrap_or(false);
                if already_attached_as_stub {
                    continue;
                }
                let short_name = helpers::short_type_name(&metadata.type_name).to_string();
                let already_listed = registered_types
                    .iter()
                    .any(|(_, existing_name, _, _)| existing_name == &short_name);
                if !already_listed {
                    registered_types.push((None, short_name, true, Some(metadata.clone())));
                }
            }

            registered_types.sort_by(|left, right| left.1.cmp(&right.1));

            if registered_types.is_empty() {
                ui.label("No registered types available.");
                return;
            }

            let mut addable_count = 0usize;
            for (type_id, name, is_addable, project_metadata) in registered_types {
                if !is_addable {
                    ui.add_enabled(
                        false,
                        egui::Button::new(format!("{name} (TODO: no runtime factory)")),
                    );
                    continue;
                }

                if let Some(project_metadata) = project_metadata.as_ref() {
                    addable_count += 1;
                    if ui.button(&name).clicked() {
                        if self.add_project_serialized_type_to_object(object_id, project_metadata) {
                            self.status_line = format!("Added {name}.");
                        } else {
                            self.status_line = format!("Failed to add {name}.");
                        }
                        ui.close();
                    }
                    continue;
                }

                let Some(type_id) = type_id else {
                    continue;
                };

                addable_count += 1;
                if ui.button(&name).clicked() {
                    if self.add_registered_type_to_object(object_id, type_id) {
                        self.status_line = format!("Added {name}.");
                    } else {
                        self.status_line = format!(
                            "Failed to add {name}: runtime registry did not provide a usable factory."
                        );
                    }
                    ui.close();
                }
            }

            if addable_count == 0 {
                ui.separator();
                ui.label("No editor-addable registered types.");
            }
        });
    }

    fn editor_settings_window(&mut self, ctx: &egui::Context) {
        if !self.editor_settings_open {
            return;
        }

        let mut open = self.editor_settings_open;
        egui::Window::new("Editor Settings")
            .open(&mut open)
            .resizable(true)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .movable(true)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.heading("Code Editor");
                property_row_like(ui, "Executable", |ui| {
                    ui.text_edit_singleline(&mut self.settings.external_editor_executable);
                });
                ui.label("Arguments");
                ui.small("Use one argument per line. Use {file} as placeholder.");
                ui.add(
                    egui::TextEdit::multiline(&mut self.settings.external_editor_args)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
                property_row_like(ui, "Presets", |ui| {
                    if ui.button("Zed").clicked() {
                        self.settings.external_editor_executable = "zed".to_string();
                        self.settings.external_editor_args = "{file}".to_string();
                    }
                    if ui.button("VS Code").clicked() {
                        self.settings.external_editor_executable = "code".to_string();
                        self.settings.external_editor_args = "--goto\n{file}".to_string();
                    }
                });

                ui.separator();
                ui.heading("Interface");
                const UI_SCALE_OPTIONS: &[(f32, &str)] = &[
                    (0.75, "75%"),
                    (0.90, "90%"),
                    (1.00, "100%"),
                    (1.10, "110%"),
                    (1.15, "115%"),
                    (1.25, "125%"),
                    (1.50, "150%"),
                    (1.75, "175%"),
                    (2.00, "200%"),
                ];
                property_row_like(ui, "UI Scale", |ui| {
                    egui::ComboBox::from_id_salt("editor_ui_scale")
                        .selected_text(
                            UI_SCALE_OPTIONS
                                .iter()
                                .find(|(value, _)| (*value - self.settings.ui_scale).abs() < 0.001)
                                .map(|(_, label)| *label)
                                .unwrap_or("100%"),
                        )
                        .show_ui(ui, |ui| {
                            for (value, label) in UI_SCALE_OPTIONS {
                                if ui
                                    .selectable_label(
                                        (self.settings.ui_scale - *value).abs() < 0.001,
                                        *label,
                                    )
                                    .clicked()
                                {
                                    self.settings.ui_scale = *value;
                                    ctx.set_zoom_factor(*value);
                                    style::apply_editor_style(ctx);
                                    ctx.request_repaint();
                                    ui.close();
                                }
                            }
                        });
                });
                property_row_like(ui, "Icon Size", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.settings.content_icon_size, 32.0..=96.0)
                            .step_by(1.0),
                    );
                });
                property_row_like(ui, "Hidden Files", |ui| {
                    if ui
                        .checkbox(&mut self.settings.show_hidden_files, "Show")
                        .changed()
                    {
                        self.content_browser.refresh(&self.settings);
                    }
                });

                ui.separator();
                if ui.button("Save Settings").clicked() {
                    match self.settings.save() {
                        Ok(()) => {
                            self.content_browser.refresh(&self.settings);
                            self.status_line = "Editor settings saved.".to_string();
                        }
                        Err(error) => {
                            self.status_line = format!("Failed to save settings: {error}");
                        }
                    }
                }
            });
        self.editor_settings_open = open;
    }

    fn project_settings_window(&mut self, ctx: &egui::Context) {
        if !self.project_settings_open {
            return;
        }

        let Some(session) = self.project_session.as_mut() else {
            self.project_settings_open = false;
            return;
        };

        let mut open = self.project_settings_open;
        egui::Window::new("Project Settings")
            .open(&mut open)
            .resizable(true)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .default_width(540.0)
            .show(ctx, |ui| {
                ui.heading("Project");
                property_row_like(ui, "Name", |ui| {
                    ui.text_edit_singleline(&mut session.project.manifest.name);
                });
                property_row_like(ui, "Binary", |ui| {
                    ui.text_edit_singleline(&mut session.project.manifest.binary_name);
                });
                property_row_like(ui, "Startup World", |ui| {
                    ui.text_edit_singleline(&mut session.project.manifest.startup_world);
                });
                property_row_like(ui, "Assets Dir", |ui| {
                    ui.text_edit_singleline(&mut session.project.manifest.assets_dir);
                });
                property_row_like(ui, "Worlds Dir", |ui| {
                    ui.text_edit_singleline(&mut session.project.manifest.worlds_dir);
                });
                property_row_like(ui, "Scripts Dir", |ui| {
                    ui.text_edit_singleline(&mut session.project.manifest.scripts_dir);
                });

                ui.separator();
                ui.heading("RunaAppConfig");
                property_row_like(ui, "Window Title", |ui| {
                    ui.text_edit_singleline(&mut session.project.manifest.app.window_title);
                });
                property_row_like(ui, "Window Size", |ui| {
                    ui.add_sized(
                        [90.0, 22.0],
                        egui::DragValue::new(&mut session.project.manifest.app.width).range(1..=8192),
                    );
                    ui.add_sized(
                        [90.0, 22.0],
                        egui::DragValue::new(&mut session.project.manifest.app.height).range(1..=8192),
                    );
                });
                property_row_like(ui, "Fullscreen", |ui| {
                    ui.checkbox(&mut session.project.manifest.app.fullscreen, "");
                });
                property_row_like(ui, "VSync", |ui| {
                    ui.checkbox(&mut session.project.manifest.app.vsync, "");
                });
                property_row_like(ui, "Show FPS", |ui| {
                    ui.checkbox(&mut session.project.manifest.app.show_fps_in_title, "");
                });
                property_row_like(ui, "Window Icon", |ui| {
                    let mut icon = session
                        .project
                        .manifest
                        .app
                        .window_icon
                        .clone()
                        .unwrap_or_default();
                    let response = ui.text_edit_singleline(&mut icon);
                    if response.changed() {
                        session.project.manifest.app.window_icon =
                            (!icon.trim().is_empty()).then_some(icon);
                    }
                });

                ui.separator();
                if ui.button("Save Project Settings").clicked() {
                    match session.project.save_manifest() {
                        Ok(()) => {
                            self.status_line = "Project settings saved.".to_string();
                        }
                        Err(error) => {
                            self.status_line =
                                format!("Failed to save project settings: {error}");
                        }
                    }
                }
            });
        self.project_settings_open = open;
    }

    fn build_settings_window(&mut self, ctx: &egui::Context) {
        if !self.build_settings_open {
            return;
        }

        let Some(session) = self.project_session.as_mut() else {
            self.build_settings_open = false;
            return;
        };

        let mut open = self.build_settings_open;
        let mut save_clicked = false;
        let mut build_clicked = false;
        egui::Window::new("Build Settings")
            .open(&mut open)
            .resizable(true)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.heading("Build");
                property_row_like(ui, "Profile", |ui| {
                    ui.selectable_value(&mut session.project.manifest.build.release, true, "Release");
                    ui.selectable_value(&mut session.project.manifest.build.release, false, "Debug");
                });
                property_row_like(ui, "Output Dir", |ui| {
                    ui.text_edit_singleline(&mut session.project.manifest.build.output_dir);
                });
                property_row_like(ui, "Hide Console", |ui| {
                    ui.checkbox(
                        &mut session.project.manifest.build.hide_console_window_on_windows,
                        "Windows release build",
                    );
                });
                ui.small("Built executable will be copied from cargo target output into the configured output directory.");

                ui.separator();
                if ui.button("Save Build Settings").clicked() {
                    save_clicked = true;
                }
                let build_enabled = self.build_process.is_none();
                let build_label = if self.build_process.is_some() {
                    "Build Running..."
                } else {
                    "Build Game"
                };
                if ui
                    .add_enabled(build_enabled, egui::Button::new(build_label))
                    .clicked()
                {
                    build_clicked = true;
                }
            });
        self.build_settings_open = open;

        if save_clicked {
            let result = ensure_release_windows_subsystem(
                &session.project.root_dir,
                session.project.manifest.build.hide_console_window_on_windows,
            )
            .and_then(|_| session.project.save_manifest());
            match result {
                Ok(()) => {
                    self.status_line = "Build settings saved.".to_string();
                }
                Err(error) => {
                    self.status_line = format!("Failed to save build settings: {error}");
                }
            }
        }
        if build_clicked {
            self.build_project();
        }
    }

    fn project_dialog_window(&mut self, ctx: &egui::Context) {
        if !self.project_dialog.open {
            return;
        }

        let mut open = self.project_dialog.open;
        egui::Window::new("New Project")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.project_dialog.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Location");
                    ui.text_edit_singleline(&mut self.project_dialog.location);
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.project_dialog.location = path.to_string_lossy().to_string();
                        }
                    }
                });

                ui.separator();
                if ui.button("Create Project").clicked() {
                    self.create_project_from_dialog();
                }
            });
        self.project_dialog.open = open;
    }

    fn viewport_settings_window(&mut self, ctx: &egui::Context) {
        if !self.view_settings_open {
            return;
        }

        let mut open = self.view_settings_open;
        egui::Window::new("View Settings")
            .open(&mut open)
            .default_width(360.0)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, [-16.0, 64.0])
            .show(ctx, |ui| {
                property_row_like(ui, "Projection", |ui| {
                    let mut projection = self.editor_camera.projection();
                    ui.selectable_value(
                        &mut projection,
                        runa_core::components::ProjectionType::Orthographic,
                        "Orthographic",
                    );
                    ui.selectable_value(
                        &mut projection,
                        runa_core::components::ProjectionType::Perspective,
                        "Perspective",
                    );
                    if projection != self.editor_camera.projection() {
                        self.editor_camera.set_projection(projection);
                    }
                });

                if self.editor_camera.is_orthographic() {
                    property_row_like(ui, "Zoom", |ui| {
                        let mut zoom = self.editor_camera.get_zoom();
                        if ui
                            .add_sized(
                                [120.0, 22.0],
                                egui::DragValue::new(&mut zoom)
                                    .speed(0.25)
                                    .range(1.0..=500.0),
                            )
                            .changed()
                        {
                            self.editor_camera.set_zoom(zoom);
                        }
                    });
                } else {
                    property_row_like(ui, "FOV", |ui| {
                        let mut fov = self.editor_camera.get_fov();
                        if ui
                            .add_sized(
                                [120.0, 22.0],
                                egui::DragValue::new(&mut fov).speed(1.0).range(20.0..=130.0),
                            )
                            .changed()
                        {
                            self.editor_camera.set_fov(fov);
                        }
                    });
                    property_row_like(ui, "Near", |ui| {
                        let mut near = self.editor_camera.near();
                        if ui
                            .add_sized(
                                [120.0, 22.0],
                                egui::DragValue::new(&mut near)
                                    .speed(0.01)
                                    .range(0.001..=999.0),
                            )
                            .changed()
                        {
                            self.editor_camera.set_near(near);
                        }
                    });
                    property_row_like(ui, "Far", |ui| {
                        let mut far = self.editor_camera.far();
                        if ui
                            .add_sized(
                                [120.0, 22.0],
                                egui::DragValue::new(&mut far).speed(1.0).range(1.0..=10000.0),
                            )
                            .changed()
                        {
                            self.editor_camera.set_far(far);
                        }
                    });
                    property_row_like(ui, "Sensitivity", |ui| {
                        let mut sensitivity = self.editor_camera.get_sensitivity();
                        if ui
                            .add_sized(
                                [120.0, 22.0],
                                egui::DragValue::new(&mut sensitivity)
                                    .speed(0.01)
                                    .range(0.01..=999.0),
                            )
                            .changed()
                        {
                            self.editor_camera.set_sensitivity(sensitivity);
                        }
                    });
                    property_row_like(ui, "Speed", |ui| {
                        let mut speed = self.editor_camera.get_speed();
                        if ui
                            .add_sized(
                                [120.0, 22.0],
                                egui::DragValue::new(&mut speed)
                                    .speed(0.05)
                                    .range(0.01..=999.0),
                            )
                            .changed()
                        {
                            self.editor_camera.set_speed(speed);
                        }
                    });
                    ui.label("Right mouse: look. WASD move, Space/Ctrl vertical, Shift boost.");
                }
            });
        self.view_settings_open = open;
    }

    fn rendering_settings_window(&mut self, ctx: &egui::Context) {
        if !self.rendering_settings_open {
            return;
        }

        let mut open = self.rendering_settings_open;
        egui::Window::new("Rendering Settings")
            .open(&mut open)
            .default_width(300.0)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, [-16.0, 64.0])
            .show(ctx, |ui| {
                ui.label("Renderer-level editor options will live here.");
            });
        self.rendering_settings_open = open;
    }

    fn gizmo_settings_window(&mut self, ctx: &egui::Context) {
        if !self.gizmo_settings_open {
            return;
        }

        let mut open = self.gizmo_settings_open;
        egui::Window::new("Gizmo")
            .open(&mut open)
            .default_width(320.0)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, [-16.0, 64.0])
            .show(ctx, |ui| {
                ui.checkbox(&mut self.gizmo_enabled, "Enabled");
                ui.checkbox(&mut self.show_viewport_grid, "Show Grid");
                ui.checkbox(&mut self.show_component_icons, "Component Icons");
                ui.checkbox(&mut self.snap_enabled, "Snap");
                property_row_like(ui, "Snap Step", |ui| {
                    ui.add_sized(
                        [120.0, 22.0],
                        egui::DragValue::new(&mut self.snap_step)
                            .speed(0.1)
                            .range(0.1..=100.0),
                    );
                });
                if self.editor_camera.is_perspective() {
                    ui.label("Transform gizmo is currently available in orthographic mode.");
                    ui.label("3D mode keeps grid + framing + component icons.");
                }
            });
        self.gizmo_settings_open = open;
    }
}

fn property_row_like(ui: &mut egui::Ui, label: &str, body: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized([110.0, 22.0], egui::Label::new(label));
        body(ui);
    });
}
