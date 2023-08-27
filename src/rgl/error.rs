use std::fmt;

#[derive(Debug)]
pub enum RglError {
    CircularProfileReference {
        profile_name: String,
    },
    Config {
        cause: Box<RglError>,
    },
    CopyDir {
        from: String,
        to: String,
        cause: Box<RglError>,
    },
    EmptyDir {
        path: String,
        cause: Box<RglError>,
    },
    ExportFailed {
        cause: Box<RglError>,
    },
    FilterConfig {
        filter_name: String,
        cause: Box<RglError>,
    },
    FilterNotDefined {
        filter_name: String,
    },
    FilterNotInstalled {
        filter_name: String,
    },
    FilterRunFailed {
        filter_name: String,
    },
    FilterTypeNotSupported {
        filter_type: String,
    },
    InvalidExportTarget {
        target: String,
    },
    InvalidFilterDefinition {
        filter_name: String,
        cause: Box<RglError>,
    },
    MoveDir {
        from: String,
        to: String,
        cause: Box<RglError>,
    },
    PathNotExists {
        path: String,
    },
    ProfileNotFound {
        profile_name: String,
    },
    ReadFile {
        path: String,
        cause: Box<RglError>,
    },
    ReadJson {
        path: String,
        cause: Box<RglError>,
    },
    Rimraf {
        path: String,
    },
    SerdeJson(serde_json::Error),
    Subprocess {
        cause: Box<RglError>,
    },
    Symlink {
        from: String,
        to: String,
        cause: Box<RglError>,
    },
    WatchDir {
        path: String,
        cause: Box<RglError>,
    },
    Wrap(Box<dyn std::error::Error>),
}

impl fmt::Display for RglError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RglError::CircularProfileReference { profile_name } => {
                write!(
                    f,
                    "<red>[+]</> Found circular profile reference in <b>{profile_name}</>"
                )
            }
            RglError::Config { cause } => {
                write!(
                    f,
                    "<red>[+]</> Could not load config.json\n\
                     {cause}"
                )
            }
            RglError::CopyDir { from, to, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to copy directory\n\
                     <yellow> >></> From: {from}\n\
                     <yellow> >></> To: {to}\n\
                     {cause}"
                )
            }
            RglError::EmptyDir { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to empty directory\n\
                     <yellow> >></> Path: {path}\n\
                     {cause}"
                )
            }
            RglError::ExportFailed { cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to export project\n\
                     {cause}"
                )
            }
            RglError::FilterConfig { filter_name, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to load config for filter <b>{filter_name}</>\n\
                     {cause}"
                )
            }
            RglError::FilterNotDefined { filter_name } => {
                write!(
                    f,
                    "<red>[+]</> Filter <b>{filter_name}</> not defined in filter_definitions"
                )
            }
            RglError::FilterNotInstalled { filter_name } => {
                write!(f, "<red>[+]</> Filter <b>{filter_name}</> not installed, run \"rgl install\" to install it")
            }
            RglError::FilterRunFailed { filter_name } => {
                write!(f, "<red>[+]</> Failed running filter <b>{filter_name}</>")
            }
            RglError::FilterTypeNotSupported { filter_type } => {
                write!(
                    f,
                    "<red>[+]</> Filter type <b>{filter_type}</> not supported"
                )
            }
            RglError::InvalidExportTarget { target } => {
                write!(f, "<red>[+]</> Export target <b>{target}</> is not valid")
            }
            RglError::InvalidFilterDefinition { filter_name, cause } => {
                write!(
                    f,
                    "<red>[+]</> Invalid filter definition for <b>{filter_name}</>\n\
                     {cause}"
                )
            }
            RglError::MoveDir { from, to, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to move directory\n\
                     <yellow> >></> From: {from}\n\
                     <yellow> >></> To: {to}\n\
                     {cause}"
                )
            }
            RglError::PathNotExists { path } => {
                write!(f, "<red>[+]</> Path <b>{path}</> does not exists")
            }
            RglError::ProfileNotFound { profile_name } => {
                write!(f, "<red>[+]</> Profile <b>{profile_name}</> not found")
            }
            RglError::ReadFile { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to read file {path}\n\
                     {cause}"
                )
            }
            RglError::ReadJson { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to load JSON file\n\
                     <yellow> >></> Path: {path}\n\
                     {cause}"
                )
            }
            RglError::Rimraf { path } => {
                write!(f, "<red>[+]</> Failed to remove directory {path}")
            }
            RglError::SerdeJson(error) => {
                write!(f, "<red>[+]</> Parse error, {error}")
            }
            RglError::Subprocess { cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed running subprocess\n\
                     {cause}"
                )
            }
            RglError::Symlink { from, to, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to create symlink\n\
                     <yellow> >></> From: {from}\n\
                     <yellow> >></> To: {to}\n\
                     {cause}"
                )
            }
            RglError::WatchDir { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to watch directory\n\
                     <yellow> >></> Path: {path}\n\
                     {cause}"
                )
            }
            RglError::Wrap(error) => {
                write!(f, "<red>[+]</> {error}")
            }
        }
    }
}

impl std::error::Error for RglError {}

pub type RglResult<T> = std::result::Result<T, RglError>;
