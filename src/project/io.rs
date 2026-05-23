use crate::project::Project;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum IoError {
    Serialize(String),
    Io(std::io::Error),
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::Serialize(e) => write!(f, "Serialization error: {}", e),
            IoError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl Error for IoError {}

impl From<ron::Error> for IoError { fn from(e: ron::Error) -> Self { IoError::Serialize(e.to_string()) } }
impl From<std::io::Error> for IoError { fn from(e: std::io::Error) -> Self { IoError::Io(e) } }

pub fn save_project(project: &Project, path: &Path) -> Result<(), IoError> {
    let ron_string = ron::to_string(project)?;
    fs::write(path, ron_string)?;
    tracing::info!("Project saved to {:?}", path);
    Ok(())
}

pub fn load_project(path: &Path) -> Result<Project, IoError> {
    let content = fs::read_to_string(path)?;
    let project: Project = ron::from_str(&content).map_err(|e| IoError::Serialize(e.to_string()))?;
    tracing::info!("Project loaded from {:?}", path);
    Ok(project)
}

pub fn save_project_pretty(project: &Project, path: &Path) -> Result<(), IoError> {
    let ron_string = ron::ser::to_string_pretty(project, ron::ser::PrettyConfig::default())?;
    fs::write(path, ron_string)?;
    tracing::info!("Project saved (pretty) to {:?}", path);
    Ok(())
}