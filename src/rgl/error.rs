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
    CurrentDirNotEmpty,
    EmptyDir {
        path: String,
        cause: Box<RglError>,
    },
    ExportFailed {
        cause: Box<RglError>,
    },
    FilterAlreadyInstalled {
        filter_name: String,
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
    FilterResolveFailed {
        filter_name: String,
    },
    FilterRunFailed {
        filter_name: String,
    },
    FilterTypeNotSupported {
        filter_type: String,
    },
    FilterVersionMismatch {
        filter_name: String,
        installed_version: String,
        required_version: String,
    },
    FilterVersionResolveFailed {
        name: String,
        url: String,
        version: String,
    },
    InvalidExportTarget {
        target: String,
    },
    InvalidFilterDefinition {
        filter_name: String,
        cause: Box<RglError>,
    },
    InvalidInstallArg {
        arg: String,
    },
    MoveDir {
        from: String,
        to: String,
        cause: Box<RglError>,
    },
    NestedRemoteFilter {
        filter_name: String,
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
    WriteFile {
        path: String,
        cause: Box<RglError>,
    },
    WriteJson {
        path: String,
        cause: Box<RglError>,
    },
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
            RglError::CurrentDirNotEmpty => {
                write!(f, "<red>[+]</> Current directory is not empty")
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
            RglError::FilterAlreadyInstalled { filter_name } => {
                write!(
                    f,
                    "Filter {filter_name} already installed, use --force to overwrite"
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
            RglError::FilterResolveFailed { filter_name } => {
                write!(
                    f,
                    "<red>[+]</> Failed to resolve filter <b>{filter_name}</>"
                )
            }
            RglError::FilterTypeNotSupported { filter_type } => {
                write!(
                    f,
                    "<red>[+]</> Filter type <b>{filter_type}</> not supported"
                )
            }
            RglError::FilterVersionMismatch {
                filter_name,
                installed_version,
                required_version,
            } => {
                write!(
                    f,
                    "<red>[+]</> Filter version mismatch\n\
                     <yellow> >></> Filter: {filter_name}\n\
                     <yellow> >></> Installed version: {installed_version}\n\
                     <yellow> >></> Required version: {required_version}"
                )
            }
            RglError::FilterVersionResolveFailed { name, url, version } => {
                write!(
                    f,
                    "<red>[+]</> Failed to resolve filter version\n\
                     <yellow> >></> Filter: {name}\n\
                     <yellow> >></> URL: {url}\n\
                     <yellow> >></> Version: {version}"
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
            RglError::InvalidInstallArg { arg } => {
                write!(f, "<red>[+]</> Invalid install argument <b>{arg}</>")
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
            RglError::NestedRemoteFilter { filter_name } => {
                write!(
                    f,
                    "<red>[+]</> Found nested remote filter definition in filter <b>{filter_name}</>\n\
                     <yellow> >></> This feature is not supported"
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
            RglError::WriteFile { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to write file\n\
                     <yellow> >></> Path: {path}\n\
                     {cause}"
                )
            }
            RglError::WriteJson { path, cause } => {
                write!(
                    f,
                    "<red>[+]</> Failed to write JSON file\n\
                     <yellow> >></> Path: {path}\n\
                     {cause}"
                )
            }
        }
    }
}

impl std::error::Error for RglError {}

pub type RglResult<T> = std::result::Result<T, RglError>;
