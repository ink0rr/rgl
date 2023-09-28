use super::{
    ref_to_version, Config, FilterDefinition, FilterInstaller, RemoteFilterDefinition, RglError,
    RglResult,
};
use semver::Version;
use simplelog::{info, warn};

pub fn install_filters(force: bool) -> RglResult<()> {
    let config = Config::load()?;
    for (name, def) in config.regolith.filter_definitions {
        if let FilterDefinition::Remote(def) = def {
            info!("Installing filter <b>{}</>...", name);
            let git_ref = match Version::parse(&def.version) {
                Ok(version) => format!("{name}-{version}"),
                Err(_) => def.version,
            };
            let filter = FilterInstaller::new(name, def.url, git_ref)?;
            if let Err(err) = filter.install(force) {
                match &err {
                    RglError::FilterAlreadyInstalled { filter_name: _ } => {
                        warn!("{err}")
                    }
                    _ => return Err(err),
                }
            }
        }
    }
    info!("Successfully installed all filters");
    Ok(())
}

pub fn install_add(filters: Vec<&String>, force: bool) -> RglResult<()> {
    let mut config = Config::load()?;
    for arg in filters {
        info!("Installing filter <b>{}</>...", arg);
        let filter = FilterInstaller::from_arg(arg)?;
        match filter.install(force) {
            Ok(_) => {
                info!("Filter <b>{}</> successfully installed", filter.name);
                let version = ref_to_version(&filter.git_ref);
                config.regolith.filter_definitions.insert(
                    filter.name,
                    FilterDefinition::Remote(RemoteFilterDefinition {
                        url: filter.url,
                        version,
                    }),
                );
            }
            Err(err) => match &err {
                RglError::FilterAlreadyInstalled { filter_name: _ } => {
                    warn!("{err}")
                }
                _ => return Err(err),
            },
        }
    }
    config.save()
}
