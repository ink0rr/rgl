use std::fmt;

#[derive(Debug)]
pub enum RglError {
    CircularProfileReferenceError {
        profile_name: String,
    },
    ConfigError {
        cause: Box<RglError>,
    },
    CopyDirError {
        from: String,
        to: String,
        cause: Box<RglError>,
    },
    EmptyDirError {
        path: String,
        cause: Box<RglError>,
    },
    ExportError {
        cause: Box<RglError>,
    },
    ExportTargetError {
        target: String,
    },
    FilterConfigError {
        filter_name: String,
        cause: Box<RglError>,
    },
    FilterNotDefinedError {
        filter_name: String,
    },
    FilterNotInstalledError {
        filter_name: String,
    },
    FilterRunError {
        filter_name: String,
    },
    FilterTypeNotSupportedError {
        filter_type: String,
    },
    InvalidFilterDefinitionError {
        filter_name: String,
        cause: Box<RglError>,
    },
    MoveError {
        from: String,
        to: String,
        cause: Box<RglError>,
    },
    ParseJsonError(serde_json::Error),
    PathNotExistsError {
        path: String,
    },
    ProfileNotFoundError {
        profile_name: String,
    },
    ReadFileError {
        path: String,
        cause: Box<RglError>,
    },
    ReadJsonError {
        path: String,
        cause: Box<RglError>,
    },
    RimrafError {
        path: String,
    },
    SubprocessError {
        cause: Box<RglError>,
    },
    SymlinkError {
        from: String,
        to: String,
        cause: Box<RglError>,
    },
    WatchError {
        path: String,
        cause: Box<RglError>,
    },
    WrapError(Box<dyn std::error::Error>),
}

impl fmt::Display for RglError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RglError::CircularProfileReferenceError { profile_name } => {
                write!(
                    f,
                    "<red>[+]</> Found circular profile reference in <b>{profile_name}</>"
                )
            }
            RglError::ConfigError { cause } => {
                write!(
                    f,
                    "<red>[+]</> Could not load config.json\n\
                     {cause}"
                )
            }
            RglError::CopyDirError { from, to, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to copy directory\n\
                     <yellow> >></> From: {from}\n\
                     <yellow> >></> To: {to}\n\
                     {cause}"
                )
            }
            RglError::EmptyDirError { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to empty directory\n\
                     <yellow> >></> Path: {path}\n\
                     {cause}"
                )
            }
            RglError::ExportError { cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to export project\n\
                     {cause}"
                )
            }
            RglError::ExportTargetError { target } => {
                write!(f, "<red>[+]</> Export target <b>{target}</> is not valid")
            }
            RglError::FilterConfigError { filter_name, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to load config for filter <b>{filter_name}</>\n\
                     {cause}"
                )
            }
            RglError::FilterNotDefinedError { filter_name } => {
                write!(
                    f,
                    "<red>[+]</> Filter <b>{filter_name}</> not defined in filter_definitions"
                )
            }
            RglError::FilterNotInstalledError { filter_name } => {
                write!(f, "<red>[+]</> Filter <b>{filter_name}</> not installed, run \"rgl install\" to install it")
            }
            RglError::FilterRunError { filter_name } => {
                write!(f, "<red>[+]</> Failed running filter <b>{filter_name}</>")
            }
            RglError::FilterTypeNotSupportedError { filter_type } => {
                write!(
                    f,
                    "<red>[+]</> Filter type <b>{filter_type}</> not supported"
                )
            }
            RglError::InvalidFilterDefinitionError { filter_name, cause } => {
                write!(
                    f,
                    "<red>[+]</> Invalid filter definition for <b>{filter_name}</>\n\
                     {cause}"
                )
            }
            RglError::MoveError { from, to, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to move files\n\
                     <yellow> >></> From: {from}\n\
                     <yellow> >></> To: {to}\n\
                     {cause}"
                )
            }
            RglError::ParseJsonError(error) => {
                write!(f, "<red>[+]</> Parse error, {error}")
            }
            RglError::PathNotExistsError { path } => {
                write!(f, "<red>[+]</> Path <b>{path}</> does not exists")
            }
            RglError::ProfileNotFoundError { profile_name } => {
                write!(f, "<red>[+]</> Profile <b>{profile_name}</> not found")
            }
            RglError::ReadFileError { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to read file {path}\n\
                     {cause}"
                )
            }
            RglError::ReadJsonError { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to load JSON file\n\
                    <yellow> >></> Path: {path}\n\
                    {cause}"
                )
            }
            RglError::RimrafError { path } => {
                write!(f, "<red>[+]</> Failed to remove directory {path}")
            }
            RglError::SubprocessError { cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed running subprocess\n\
                     {cause}"
                )
            }
            RglError::SymlinkError { from, to, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to create symlink\n\
                     <yellow> >></> From: {from}\n\
                     <yellow> >></> To: {to}\n\
                     {cause}"
                )
            }
            RglError::WatchError { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to watch directory\n\
                     <yellow> >></> Path: {path}\n\
                     {cause}"
                )
            }
            RglError::WrapError(error) => {
                write!(f, "<red>[+]</> {error}")
            }
        }
    }
}

impl std::error::Error for RglError {}

pub type RglResult<T> = std::result::Result<T, RglError>;
