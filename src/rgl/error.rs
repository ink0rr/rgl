use std::fmt;

#[derive(Debug)]
pub enum RglError {
    CircularProfileReferenceError(String),
    ConfigError,
    CopyError(String, String, String),
    EmptyDirError(String, String),
    ExportError,
    ExportTargetError(String),
    FilterConfigError(String),
    FilterNotDefinedError(String),
    FilterNotInstalledError(String),
    FilterNotSupportedError(String),
    FilterRunError(String),
    InvalidFilterDefinitionError(String),
    MoveError(String, String, String),
    ParseJsonError(serde_json::Error),
    PathNotExistsError(String),
    ProfileNotFoundError(String),
    ReadFileError(String),
    ReadJsonError(String, serde_json::Error),
    RimrafError(String),
    SubprocessError(String),
    SymlinkError(String, String, String),
    WatchError(String, String),
    WrappedError(WrappedErrorContent),
    WriteFileError(String),
}

#[derive(Debug)]
pub struct WrappedErrorContent {
    pub root: Box<RglError>,
    pub cause: Box<RglError>,
}

impl fmt::Display for RglError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RglError::CircularProfileReferenceError(profile) => {
                write!(
                    f,
                    "<red>[+]</> Found circular profile reference in <b>{profile}</>"
                )
            }
            RglError::ConfigError => {
                write!(f, "<red>[+]</> Failed to load config file")
            }
            RglError::CopyError(from, to, cause) => {
                write!(
                    f,
                    "<red>[+]</> Failed to copy files\n\
                    \tFrom: {from}\n\
                    \tTo: {to}\n\
                    <yellow> >></> Cause: {cause}"
                )
            }
            RglError::EmptyDirError(path, cause) => {
                write!(
                    f,
                    "<red>[+]</> Failed to empty directory\n\
                    <yellow> >></> Path: {path}\n\
                    <red>[+]</> {cause}"
                )
            }
            RglError::ExportError => {
                write!(f, "<red>[+]</> Failed to export project")
            }
            RglError::ExportTargetError(target) => {
                write!(f, "<red>[+]</> Invalid export target <b>{target}</>")
            }
            RglError::FilterConfigError(filter) => {
                write!(
                    f,
                    "<red>[+]</> Failed to load config for filter <b>{filter}</>"
                )
            }
            RglError::FilterNotDefinedError(filter) => {
                write!(
                    f,
                    "<red>[+]</> Filter <b>{filter}</> not defined in filter_definitions"
                )
            }
            RglError::FilterNotInstalledError(filter) => {
                write!(f, "<red>[+]</> Filter <b>{filter}</> not installed, run \"rgl install\" to install it")
            }
            RglError::FilterNotSupportedError(filter) => {
                write!(f, "<red>[+]</> Filter <b>{filter}</> not supported")
            }
            RglError::FilterRunError(filter) => {
                write!(f, "<red>[+]</> Failed to run filter <b>{filter}</>")
            }
            RglError::InvalidFilterDefinitionError(filter) => {
                write!(
                    f,
                    "<red>[+]</> Invalid filter definition for <b>{filter}</>"
                )
            }
            RglError::MoveError(from, to, cause) => {
                write!(
                    f,
                    "<red>[+]</> Failed to move files\n\
                    \tFrom: {from}\n\
                    \tTo: {to}\n\
                    <yellow> >></> Cause: {cause}"
                )
            }
            RglError::ParseJsonError(cause) => {
                write!(
                    f,
                    "<red>[+]</> Failed to parse json\n\
                    <red>[+]</> {cause}"
                )
            }
            RglError::PathNotExistsError(path) => {
                write!(f, "<red>[+]</> Path <b>{path}</> does not exists")
            }
            RglError::ProfileNotFoundError(profile) => {
                write!(f, "<red>[+]</> Profile <b>{profile}</> not found")
            }
            RglError::ReadFileError(path) => {
                write!(f, "<red>[+]</> Failed to read file {path}")
            }
            RglError::ReadJsonError(path, cause) => {
                write!(
                    f,
                    "<red>[+]</> Failed to parse json file\n\
                    <yellow> >></> Path: {path}\n\
                    <red>[+]</> {cause}",
                )
            }
            RglError::RimrafError(path) => {
                write!(f, "<red>[+]</> Failed to remove directory {path}")
            }
            RglError::SubprocessError(cause) => {
                write!(
                    f,
                    "<red>[+]</> Failed running subprocess\n\
                    <red>[+]</> {cause}"
                )
            }
            RglError::SymlinkError(from, to, cause) => {
                write!(
                    f,
                    "<red>[+]</> Failed to create symlink\n\
                    \tFrom: {from}\n\
                    \tTo: {to}\n\
                    <yellow> >></> Cause: {cause}"
                )
            }
            RglError::WatchError(path, cause) => {
                write!(
                    f,
                    "<red>[+]</> Failed to watch directory\n\
                    <yellow> >></> Path: {path}\n\
                    <red>[+]</> {cause}"
                )
            }
            RglError::WrappedError(e) => {
                write!(f, "{}\n{}", e.root, e.cause)
            }
            RglError::WriteFileError(path) => {
                write!(
                    f,
                    "<red>[+]</> Failed to write file\n\
                    <yellow> >></> Path: {path}"
                )
            }
        }
    }
}

impl std::error::Error for RglError {}

pub type Result<T> = std::result::Result<T, RglError>;
