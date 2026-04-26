use super::*;

impl<'window> EditorApp<'window> {
    pub(super) fn sync_world_serialized_type_metadata(&mut self) {
        let registered_types = self.place_object.registered_types.clone();
        if registered_types.is_empty() {
            return;
        }

        let object_ids = self.world_object_ids();
        for object_id in object_ids {
            let Some(object) = self.world.get_mut(object_id) else {
                continue;
            };
            let Some(storage) = object.get_component_mut::<SerializedTypeStorage>() else {
                continue;
            };

            for entry in &mut storage.entries {
                let expected_kind = match entry.kind {
                    SerializedTypeKind::Component => ProjectRegisteredTypeKind::Component,
                    SerializedTypeKind::Script => ProjectRegisteredTypeKind::Script,
                };
                let Some(metadata) = registered_types.iter().find(|metadata| {
                    metadata.kind == expected_kind
                        && (metadata.type_name == entry.type_name
                            || short_type_name(&metadata.type_name)
                                == short_type_name(&entry.type_name))
                }) else {
                    continue;
                };

                entry.type_name = metadata.type_name.clone();

                let mut merged_fields = Vec::with_capacity(metadata.default_fields.len());
                for default_field in &metadata.default_fields {
                    if let Some(existing) = entry
                        .fields
                        .iter()
                        .find(|field| field.name == default_field.name)
                    {
                        merged_fields.push(existing.clone());
                    } else {
                        merged_fields.push(default_field.clone());
                    }
                }
                entry.fields = merged_fields;
            }
        }
    }

    pub(super) fn add_registered_type_to_object(
        &mut self,
        object_id: ObjectId,
        type_id: TypeId,
    ) -> bool {
        let registry = self.runtime_registry().clone();
        let Some(object) = self.world.get_mut(object_id) else {
            return false;
        };
        registry.add_type_to_object(object, type_id)
    }

    pub(super) fn add_project_serialized_type_to_object(
        &mut self,
        object_id: ObjectId,
        metadata: &ProjectRegisteredTypeRecord,
    ) -> bool {
        let Some(object) = self.world.get_mut(object_id) else {
            return false;
        };

        let kind = match metadata.kind {
            ProjectRegisteredTypeKind::Component => SerializedTypeKind::Component,
            ProjectRegisteredTypeKind::Script => SerializedTypeKind::Script,
        };
        let mut storage = object
            .get_component::<SerializedTypeStorage>()
            .cloned()
            .unwrap_or_default();
        storage.upsert(SerializedTypeEntry {
            type_name: metadata.type_name.clone(),
            kind,
            fields: metadata.default_fields.clone(),
        });
        if object.get_component::<SerializedTypeStorage>().is_some() {
            object.remove_component_type_id(TypeId::of::<SerializedTypeStorage>());
        }
        object.add_component(storage);
        true
    }

    pub(super) fn create_empty_object(&mut self) {
        let object_id = self.world.spawn(Object::new("Empty"));
        self.set_primary_selection(Some(object_id));
        self.status_line = "Created empty object.".to_string();
    }

    pub(super) fn create_empty_child_object(&mut self, parent_id: ObjectId) {
        let object_id = self.world.spawn(Object::new("Empty"));
        if self.world.set_parent(object_id, Some(parent_id)) {
            self.hierarchy_expanded.insert(parent_id);
        }
        self.set_primary_selection(Some(object_id));
        self.status_line = "Created child object.".to_string();
    }

    pub(super) fn create_from_archetype_menu_ui(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Create From Archetype", |ui| {
            self.refresh_project_metadata(false);
            self.poll_place_object_refresh();

            let mut archetypes = self.runtime_registry().archetypes().metadata();
            archetypes.sort_by(|left, right| left.name().cmp(right.name()));
            let mut project_archetypes: Vec<PlaceableObjectDescriptor> = self
                .place_object
                .objects
                .iter()
                .filter(|object| object.category == "Archetypes")
                .cloned()
                .collect();
            project_archetypes.sort_by(|left, right| left.name.cmp(&right.name));

            if archetypes.is_empty() && project_archetypes.is_empty() {
                ui.label("No registered archetypes.");
                return;
            }

            let has_runtime_archetypes = !archetypes.is_empty();
            for archetype in &archetypes {
                let name = archetype.name().to_string();
                let key = archetype.key().clone();
                if ui.button(&name).clicked() {
                    if let Some(object_id) = self.world.spawn_archetype_by_key(&key) {
                        self.set_primary_selection(Some(object_id));
                        self.status_line = format!("Created object from archetype {name}.");
                    } else {
                        self.status_line =
                            format!("Failed to create object from archetype {name}.");
                    }
                    ui.close();
                }
            }

            if !project_archetypes.is_empty() {
                if has_runtime_archetypes {
                    ui.separator();
                }
                for archetype in project_archetypes {
                    let name = archetype.name.clone();
                    if ui.button(&name).clicked() {
                        self.place_object(&archetype);
                        self.status_line = format!("Created object from project archetype {name}.");
                        ui.close();
                    }
                }
            }
        });
    }

    pub(super) fn ensure_world_runtime_registry(&mut self) {
        if self.world.runtime_registry().is_none() {
            self.world
                .set_runtime_registry(Arc::new(self.runtime_engine.runtime_registry().clone()));
        }
        self.world.refresh_object_world_ptrs();
    }

    pub(super) fn runtime_registry(&self) -> &RuntimeRegistry {
        self.world
            .runtime_registry()
            .unwrap_or_else(|| self.runtime_engine.runtime_registry())
    }

    pub(super) fn copy_object(&mut self, object_id: ObjectId, cut: bool) {
        let Some(object) = self.world.get(object_id) else {
            return;
        };
        self.hierarchy_clipboard = Some(ObjectClipboard {
            asset: WorldObjectAsset::from_object(object),
            cut_id: cut.then_some(object_id),
        });
        self.status_line = if cut {
            "Cut object.".to_string()
        } else {
            "Copied object.".to_string()
        };
    }

    pub(super) fn paste_object(&mut self, target_id: Option<ObjectId>) {
        let Some(clipboard) = self.hierarchy_clipboard.take() else {
            return;
        };
        let project_root = self
            .project_session
            .as_ref()
            .map(|session| session.project.root_dir.as_path());
        let mut object = clipboard.asset.clone().into_object(project_root);
        if let Some(object_id) = clipboard.asset.object_id.clone() {
            if object.get_component::<ObjectDefinitionInstance>().is_none() {
                object.add_component(ObjectDefinitionInstance::new(object_id));
            }
        }

        if let Some(cut_id) = clipboard.cut_id {
            self.world.despawn(cut_id);
        }
        let new_id = self.world.spawn(object);
        if let Some(target_id) = target_id {
            self.world.set_parent(new_id, Some(target_id));
        }
        self.set_primary_selection(Some(new_id));
        self.status_line = "Pasted object.".to_string();
    }

    pub(super) fn delete_object(&mut self, object_id: ObjectId) {
        self.world.despawn(object_id);
        self.set_primary_selection(self.first_object_id());
        self.status_line = "Deleted object.".to_string();
    }

    pub(super) fn place_object_menu_ui(&mut self, ui: &mut egui::Ui) {
        if self.project_session.is_none() {
            ui.label("Open a project to use Place Object.");
            return;
        }

        self.refresh_project_metadata(false);
        self.poll_place_object_refresh();

        if ui.button("Refresh Project Metadata").clicked() {
            self.refresh_project_metadata(true);
            ui.close();
            return;
        }

        if self.place_object.refresh_in_progress {
            ui.separator();
            ui.label("Refreshing objects...");
            return;
        }

        if self.place_object.objects.is_empty() {
            ui.separator();
            ui.label("No placeable objects found.");
            return;
        }

        ui.separator();
        let mut categories: Vec<String> = self
            .place_object
            .objects
            .iter()
            .map(|object| object.category.clone())
            .collect();
        categories.sort();
        categories.dedup();

        for category in categories {
            ui.menu_button(category.clone(), |ui| {
                let matching_objects: Vec<PlaceableObjectDescriptor> = self
                    .place_object
                    .objects
                    .iter()
                    .filter(|object| object.category == category)
                    .cloned()
                    .collect();
                for object in matching_objects {
                    if ui.button(&object.name).clicked() {
                        self.place_object(&object);
                        ui.close();
                    }
                }
            });
        }
    }

    pub(super) fn place_object(&mut self, object: &PlaceableObjectDescriptor) {
        let Some(mut asset) = self.place_object.templates.get(&object.id).cloned() else {
            self.status_line = format!("Failed to spawn object {}.", object.name);
            return;
        };

        asset.object_id = Some(object.id.clone());
        let project_root = self
            .project_session
            .as_ref()
            .map(|session| session.project.root_dir.as_path());
        let mut world_object = asset.into_object(project_root);
        if world_object
            .get_component::<ObjectDefinitionInstance>()
            .is_none()
        {
            world_object.add_component(ObjectDefinitionInstance::new(object.id.clone()));
        }

        let object_id = self.world.spawn(world_object);
        self.set_primary_selection(Some(object_id));
        self.status_line = format!("Placed object {}.", object.name);
    }

    pub(super) fn refresh_placeable_objects_if_needed(&mut self, force: bool) {
        let Some(session) = self.project_session.as_ref().cloned() else {
            return;
        };

        self.poll_place_object_refresh();
        if self.place_object.refresh_in_progress {
            return;
        }
        let latest_stamp = placeables::latest_place_object_stamp(&session.project);
        if !force && latest_stamp == self.place_object.source_stamp {
            return;
        }
        if let Err(error) = ensure_editor_bridge_files(&session.project.root_dir) {
            self.status_line = format!("Failed to refresh project bridge files: {error}");
            self.push_output(self.status_line.clone());
            return;
        }
        let latest_stamp = placeables::latest_place_object_stamp(&session.project);

        let project = session.project.clone();
        let output_tx = self.output_tx.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(placeables::query_project_metadata(&project, &output_tx));
        });
        self.place_object.pending_stamp = latest_stamp;
        self.place_object.refresh_in_progress = true;
        self.place_object.refresh_result = Some(rx);
    }

    pub(super) fn poll_place_object_refresh(&mut self) {
        let Some(receiver) = self.place_object.refresh_result.as_ref() else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => {
                self.place_object.refresh_result = None;
                self.place_object.refresh_in_progress = false;
                match result {
                    Ok(metadata) => {
                        let merged_records =
                            placeables::merge_placeable_object_records(metadata.object_records);
                        self.place_object.objects = merged_records
                            .iter()
                            .map(|record| record.descriptor.clone())
                            .collect();
                        self.place_object.templates = merged_records
                            .into_iter()
                            .map(|record| (record.descriptor.id.clone(), record.object))
                            .collect();
                        self.place_object.registered_types = metadata.registered_types;
                        self.place_object.source_stamp = self.place_object.pending_stamp.take();
                        self.sync_world_serialized_type_metadata();
                        self.status_line = "Project metadata refreshed.".to_string();
                    }
                    Err(error) => {
                        self.place_object.pending_stamp = None;
                        self.status_line = format!("Failed to refresh placeable objects: {error}");
                    }
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.place_object.refresh_result = None;
                self.place_object.refresh_in_progress = false;
                self.place_object.pending_stamp = None;
                self.status_line =
                    "Object refresh failed because the background job disconnected.".to_string();
            }
        }
    }
}

fn short_type_name(type_name: &str) -> &str {
    type_name.rsplit("::").next().unwrap_or(type_name)
}
