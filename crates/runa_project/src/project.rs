use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ProjectError {
    Io(std::io::Error),
    Ron(ron::Error),
    RonSpanned(ron::error::SpannedError),
    Message(String),
}

impl Display for ProjectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Ron(error) => write!(f, "{error}"),
            Self::RonSpanned(error) => write!(f, "{error}"),
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}

impl Error for ProjectError {}

impl From<std::io::Error> for ProjectError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ron::Error> for ProjectError {
    fn from(value: ron::Error) -> Self {
        Self::Ron(value)
    }
}

impl From<ron::error::SpannedError> for ProjectError {
    fn from(value: ron::error::SpannedError) -> Self {
        Self::RonSpanned(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    pub engine_version: String,
    pub startup_world: String,
    pub assets_dir: String,
    pub worlds_dir: String,
    pub scripts_dir: String,
    pub binary_name: String,
    #[serde(default)]
    pub app: ProjectAppConfig,
    #[serde(default)]
    pub build: ProjectBuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAppConfig {
    pub window_title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub show_fps_in_title: bool,
    pub window_icon: Option<String>,
}

impl Default for ProjectAppConfig {
    fn default() -> Self {
        Self {
            window_title: "Runa Game".to_string(),
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: true,
            show_fps_in_title: false,
            window_icon: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBuildConfig {
    pub output_dir: String,
    pub release: bool,
    pub hide_console_window_on_windows: bool,
}

impl Default for ProjectBuildConfig {
    fn default() -> Self {
        Self {
            output_dir: "build".to_string(),
            release: true,
            hide_console_window_on_windows: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectPaths {
    pub manifest_path: PathBuf,
    pub root_dir: PathBuf,
    pub manifest: ProjectManifest,
}

impl ProjectPaths {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let manifest_path = resolve_manifest_path(path.as_ref())?;
        let root_dir = manifest_path
            .parent()
            .ok_or_else(|| {
                ProjectError::Message("Project manifest has no parent directory.".to_string())
            })?
            .to_path_buf();
        let content = fs::read_to_string(&manifest_path)?;
        let manifest: ProjectManifest = ron::from_str(&content)?;
        Ok(Self {
            manifest_path,
            root_dir,
            manifest,
        })
    }

    pub fn save_manifest(&self) -> Result<(), ProjectError> {
        let content =
            ron::ser::to_string_pretty(&self.manifest, ron::ser::PrettyConfig::default())?;
        fs::write(&self.manifest_path, content)?;
        Ok(())
    }

    pub fn startup_world_path(&self) -> PathBuf {
        self.root_dir.join(&self.manifest.startup_world)
    }

    pub fn worlds_dir(&self) -> PathBuf {
        self.root_dir.join(&self.manifest.worlds_dir)
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.root_dir.join(&self.manifest.assets_dir)
    }

    pub fn scripts_dir(&self) -> PathBuf {
        self.root_dir.join(&self.manifest.scripts_dir)
    }
}

pub fn find_project_manifest(start: impl AsRef<Path>) -> Option<PathBuf> {
    let mut current = start.as_ref();
    if current.is_file() {
        current = current.parent()?;
    }

    loop {
        if let Ok(read_dir) = fs::read_dir(current) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("runaproj") {
                    return Some(path);
                }
            }
        }

        current = current.parent()?;
    }
}

pub fn load_project(path: impl AsRef<Path>) -> Result<ProjectPaths, ProjectError> {
    ProjectPaths::load(path)
}

fn resolve_manifest_path(path: &Path) -> Result<PathBuf, ProjectError> {
    if path.is_file() {
        if path.extension().and_then(|ext| ext.to_str()) == Some("runaproj") {
            return Ok(path.to_path_buf());
        }
        return Err(ProjectError::Message(format!(
            "Expected a .runaproj file, got {}",
            path.display()
        )));
    }

    find_project_manifest(path).ok_or_else(|| {
        ProjectError::Message(format!(
            "No .runaproj manifest found in {} or its parent directories.",
            path.display()
        ))
    })
}
