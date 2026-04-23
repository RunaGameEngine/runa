use super::*;

impl<'window> EditorApp<'window> {
    pub(super) fn refresh_project_metadata(&mut self, force: bool) {
        let Some(session) = self.project_session.as_ref() else {
            if force {
                self.status_line = "Open a project before refreshing project metadata.".to_string();
            }
            return;
        };

        if force {
            self.content_browser.refresh(&self.settings);
            self.status_line = format!(
                "Refreshing project metadata for {}...",
                session.project.manifest.name
            );
            self.push_output(self.status_line.clone());
        }

        self.refresh_placeable_objects_if_needed(force);
    }

    pub(super) fn new_world(&mut self) {
        self.world = create_empty_world();
        self.ensure_world_runtime_registry();
        self.selection = self.first_object_id();
        if let Some(session) = self.project_session.as_mut() {
            session.current_world_path = None;
        }
        self.status_line = "Created a new world.".to_string();
    }

    pub(super) fn open_project_dialog(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Runa Project", &["runaproj"])
            .pick_file()
        {
            self.begin_load_project(path);
        }
    }

    pub(super) fn begin_load_project(&mut self, path: PathBuf) {
        let output_tx = self.output_tx.clone();
        let (tx, rx) = mpsc::channel();
        self.project_load = Some(rx);
        self.status_line = format!("Opening project {}...", path.display());
        self.push_output(format!("Opening project {}...", path.display()));

        std::thread::spawn(move || {
            let _ = output_tx.send(format!("Loading project manifest: {}", path.display()));
            let result = (|| -> Result<ProjectLoadResult, String> {
                let project = load_project(&path).map_err(|error| error.to_string())?;
                ensure_editor_bridge_files(&project.root_dir).map_err(|error| error.to_string())?;
                let metadata = placeables::query_project_metadata(&project, &output_tx)?;

                Ok(ProjectLoadResult { project, metadata })
            })();
            let _ = tx.send(result);
        });
    }

    pub(super) fn apply_loaded_project(&mut self, result: ProjectLoadResult) {
        let startup_world_path = result.project.startup_world_path();
        let world = if startup_world_path.exists() {
            match load_world(&startup_world_path) {
                Ok(world) => world,
                Err(error) => {
                    self.push_output(format!(
                        "Failed to load startup world: {error}. Using an empty world instead."
                    ));
                    create_empty_world()
                }
            }
        } else {
            create_empty_world()
        };
        self.project_session = Some(ProjectSession {
            current_world_path: Some(startup_world_path),
            project: result.project.clone(),
        });
        self.world = world;
        self.ensure_world_runtime_registry();
        self.selection = self.first_object_id();
        self.content_browser
            .set_project_root(result.project.root_dir.clone(), &self.settings);
        let merged_records = placeables::merge_placeable_object_records(result.metadata.object_records);
        self.place_object.objects = merged_records
            .iter()
            .map(|record| record.descriptor.clone())
            .collect();
        self.place_object.templates = merged_records
            .into_iter()
            .map(|record| (record.descriptor.id.clone(), record.object))
            .collect();
        self.place_object.registered_types = result.metadata.registered_types;
        self.place_object.source_stamp = None;
        self.status_line = format!("Opened project {}.", result.project.manifest.name);
        self.push_output(self.status_line.clone());
    }

    pub(super) fn open_world_dialog(&mut self) {
        let start_dir = self
            .project_session
            .as_ref()
            .map(|session| session.project.worlds_dir())
            .unwrap_or_else(helpers::default_browse_root);
        if let Some(path) = FileDialog::new()
            .set_directory(start_dir)
            .add_filter("Runa World", &["ron"])
            .pick_file()
        {
            self.open_world_from_path(path);
        }
    }

    pub(super) fn open_world_from_path(&mut self, path: PathBuf) {
        match load_world(&path) {
            Ok(world) => {
                self.world = world;
                self.ensure_world_runtime_registry();
                self.selection = self.first_object_id();
                if let Some(session) = self.project_session.as_mut() {
                    session.current_world_path = Some(path.clone());
                }
                self.status_line = format!("Opened world {}.", path.display());
            }
            Err(error) => {
                self.status_line = format!("Failed to open world: {error}");
            }
        }
    }

    pub(super) fn save_current_world(&mut self) {
        if let Some(path) = self.current_world_path() {
            self.save_world_to_path(path);
        } else {
            self.save_world_as_dialog();
        }
    }

    pub(super) fn save_world_as_dialog(&mut self) {
        let suggested_dir = self
            .project_session
            .as_ref()
            .map(|session| session.project.worlds_dir())
            .unwrap_or_else(helpers::default_browse_root);
        if let Some(path) = FileDialog::new()
            .set_directory(suggested_dir)
            .set_file_name("main.world.ron")
            .add_filter("Runa World", &["ron"])
            .save_file()
        {
            self.save_world_to_path(helpers::ensure_world_extension(path));
        }
    }

    pub(super) fn save_world_to_path(&mut self, path: PathBuf) {
        match save_world(&path, &self.world) {
            Ok(()) => {
                if let Some(session) = self.project_session.as_mut() {
                    session.current_world_path = Some(path.clone());
                    if let Ok(relative) = path.strip_prefix(&session.project.root_dir) {
                        session.project.manifest.startup_world =
                            relative.to_string_lossy().replace('\\', "/");
                        if let Err(error) = session.project.save_manifest() {
                            self.status_line =
                                format!("World saved but failed to update startup world: {error}");
                            return;
                        }
                    }
                }
                self.status_line = format!("Saved world to {}.", path.display());
            }
            Err(error) => {
                self.status_line = format!("Failed to save world: {error}");
            }
        }
    }

    pub(super) fn play_project(&mut self) {
        let Some(session) = self.project_session.as_ref().cloned() else {
            self.status_line = "Open a project before starting Play mode.".to_string();
            return;
        };

        if self.runtime_process.is_some() {
            self.stop_project();
        }

        self.save_current_world();

        let mut command = Command::new("cargo");
        command
            .args(["run", "--bin", &session.project.manifest.binary_name])
            .current_dir(&session.project.root_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        placeables::configure_background_command(&mut command);

        match command.spawn() {
            Ok(child) => {
                self.runtime_process = Some(child);
                if let Some(child_ref) = self.runtime_process.as_mut() {
                    placeables::attach_child_output(child_ref, self.output_tx.clone(), "play");
                }
                self.status_line =
                    format!("Started Play mode for {}.", session.project.manifest.name);
                self.push_output(self.status_line.clone());
            }
            Err(error) => {
                self.status_line = format!("Failed to start Play mode: {error}");
                self.push_output(self.status_line.clone());
            }
        }
    }

    pub(super) fn stop_project(&mut self) {
        let Some(mut child) = self.runtime_process.take() else {
            return;
        };

        match child.kill() {
            Ok(()) => {
                let _ = child.wait();
                self.status_line = "Stopped Play mode.".to_string();
                self.push_output(self.status_line.clone());
            }
            Err(error) => {
                self.status_line = format!("Failed to stop Play mode: {error}");
                self.push_output(self.status_line.clone());
            }
        }
    }

    pub(super) fn update_runtime_process_state(&mut self) {
        let Some(child) = self.runtime_process.as_mut() else {
            return;
        };

        match child.try_wait() {
            Ok(Some(status)) => {
                self.runtime_process = None;
                self.status_line = format!("Play mode exited with status {status}.");
                self.push_output(self.status_line.clone());
            }
            Ok(None) => {}
            Err(error) => {
                self.runtime_process = None;
                self.status_line = format!("Failed to poll Play mode: {error}");
                self.push_output(self.status_line.clone());
            }
        }
    }

    pub(super) fn build_project(&mut self) {
        let Some(session) = self.project_session.as_ref().cloned() else {
            self.status_line = "Open a project before building.".to_string();
            return;
        };

        if self.build_process.is_some() {
            self.status_line = "Build is already running.".to_string();
            return;
        }

        if let Err(error) = ensure_release_windows_subsystem(
            &session.project.root_dir,
            session.project.manifest.build.hide_console_window_on_windows,
        ) {
            self.status_line = format!("Failed to prepare release main.rs: {error}");
            self.push_output(self.status_line.clone());
            return;
        }

        let release = session.project.manifest.build.release;
        let profile_label = if release { "release" } else { "debug" };
        let mut command = Command::new("cargo");
        command
            .arg("build")
            .arg("--bin")
            .arg(&session.project.manifest.binary_name)
            .current_dir(&session.project.root_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if release {
            command.arg("--release");
        }
        placeables::configure_background_command(&mut command);

        match command.spawn() {
            Ok(child) => {
                self.build_process = Some(child);
                if let Some(child_ref) = self.build_process.as_mut() {
                    placeables::attach_child_output(child_ref, self.output_tx.clone(), "build");
                }
                self.status_line = format!("Started {profile_label} build.");
                self.push_output(self.status_line.clone());
            }
            Err(error) => {
                self.status_line = format!("Failed to start build: {error}");
                self.push_output(self.status_line.clone());
            }
        }
    }

    pub(super) fn update_build_process_state(&mut self) {
        let Some(child) = self.build_process.as_mut() else {
            return;
        };

        match child.try_wait() {
            Ok(Some(status)) => {
                let Some(session) = self.project_session.as_ref().cloned() else {
                    self.build_process = None;
                    return;
                };
                self.build_process = None;

                if !status.success() {
                    self.status_line = format!("Build failed with status {status}.");
                    self.push_output(self.status_line.clone());
                    return;
                }

                let profile_dir = if session.project.manifest.build.release {
                    "release"
                } else {
                    "debug"
                };
                let executable_name = if cfg!(target_os = "windows") {
                    format!("{}.exe", session.project.manifest.binary_name)
                } else {
                    session.project.manifest.binary_name.clone()
                };
                let source_binary = session
                    .project
                    .root_dir
                    .join("target")
                    .join(profile_dir)
                    .join(&executable_name);
                let output_dir = session
                    .project
                    .root_dir
                    .join(&session.project.manifest.build.output_dir);
                let destination_binary = output_dir.join(&executable_name);

                match std::fs::create_dir_all(&output_dir)
                    .and_then(|_| std::fs::copy(&source_binary, &destination_binary).map(|_| ()))
                {
                    Ok(()) => {
                        self.status_line =
                            format!("Build finished: {}", destination_binary.display());
                        self.push_output(self.status_line.clone());
                    }
                    Err(error) => {
                        self.status_line = format!(
                            "Build finished but failed to copy artifact: {error}"
                        );
                        self.push_output(self.status_line.clone());
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                self.build_process = None;
                self.status_line = format!("Failed to poll build process: {error}");
                self.push_output(self.status_line.clone());
            }
        }
    }

    pub(super) fn poll_project_load(&mut self) {
        let Some(receiver) = self.project_load.as_ref() else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => {
                self.project_load = None;
                match result {
                    Ok(result) => self.apply_loaded_project(result),
                    Err(error) => {
                        self.status_line = format!("Failed to open project: {error}");
                        self.push_output(self.status_line.clone());
                    }
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.project_load = None;
                self.status_line = "Project loading task disconnected.".to_string();
                self.push_output(self.status_line.clone());
            }
        }
    }

    pub(super) fn poll_output(&mut self) {
        while let Ok(line) = self.output_rx.try_recv() {
            self.output_lines.push(line);
            if self.output_lines.len() > 500 {
                let drain_len = self.output_lines.len() - 500;
                self.output_lines.drain(0..drain_len);
            }
        }
    }

    pub(super) fn push_output(&mut self, line: impl Into<String>) {
        self.output_lines.push(line.into());
        if self.output_lines.len() > 500 {
            let drain_len = self.output_lines.len() - 500;
            self.output_lines.drain(0..drain_len);
        }
    }

    pub(super) fn current_world_path(&self) -> Option<PathBuf> {
        self.project_session
            .as_ref()
            .and_then(|session| session.current_world_path.clone())
    }

    pub(super) fn window_title(&self) -> String {
        if let Some(session) = self.project_session.as_ref() {
            format!("Runa Editor - {}", session.project.manifest.name)
        } else {
            "Runa Editor".to_string()
        }
    }

    pub(super) fn create_project_from_dialog(&mut self) {
        let name = self.project_dialog.name.trim();
        let location = self.project_dialog.location.trim();
        if name.is_empty() || location.is_empty() {
            self.status_line = "Project name and location are required.".to_string();
            return;
        }

        let destination = PathBuf::from(location).join(name);
        match create_empty_project(&destination, name) {
            Ok(project) => {
                self.project_dialog.open = false;
                self.begin_load_project(project.manifest_path.clone());
                self.status_line = format!("Created project {}.", project.manifest.name);
            }
            Err(error) => {
                self.status_line = format!("Failed to create project: {error}");
            }
        }
    }
}
