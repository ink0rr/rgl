use super::{
    copy_dir, empty_dir, get_filter_cache_dir, read_json, resolve_url, rimraf, write_json,
    FilterRemote, RglError, RglResult, Subprocess,
};
use semver::Version;
use serde_json::Value;
use simplelog::info;
use std::path::Path;

pub struct FilterInstaller {
    pub name: String,
    pub url: String,
    pub git_ref: String,
}

impl FilterInstaller {
    pub fn new(name: String, url: String, git_ref: String) -> RglResult<Self> {
        Ok(Self { name, url, git_ref })
    }

    pub fn from_arg(arg: &str) -> RglResult<Self> {
        let name: String;
        let mut url: String;
        let version: Option<String>;

        if arg.contains("==") {
            let parts: Vec<_> = arg.split("==").collect();
            if parts.len() != 2 {
                return Err(RglError::InvalidInstallArg {
                    arg: arg.to_owned(),
                });
            }
            url = parts[0].to_owned();
            version = Some(parts[1].to_owned());
        } else {
            url = arg.to_owned();
            version = None;
        }

        if url.contains("/") {
            let mut split: Vec<_> = url.split("/").collect();
            name = split.pop().unwrap().to_owned();
            url = split.join("/");
            if url.starts_with("https://") {
                url = url.replace("https://", "");
            }
        } else {
            name = url;
            url = resolve_url(&name)?;
        }

        let git_ref = get_git_ref(&name, &url, version)?;
        info!("Resolved <b>{arg}</> to <b>{url}@{git_ref}</>");

        Ok(Self { name, url, git_ref })
    }

    pub fn install(&self, force: bool) -> RglResult<()> {
        let filter_dir = Path::new(".regolith")
            .join("cache")
            .join("filters")
            .join(&self.name);

        if filter_dir.exists() && !force {
            return Err(RglError::FilterAlreadyInstalled {
                filter_name: self.name.to_owned(),
            });
        } else {
            rimraf(&filter_dir)?;
        }

        let https_url = format!("https://{}", self.url);
        let cache_dir = get_filter_cache_dir(&https_url);

        if !cache_dir.exists() {
            empty_dir(&cache_dir)?;
            Subprocess::new("git")
                .args(vec!["clone", &https_url, "."])
                .current_dir(&cache_dir)
                .run_silent()?;
        }

        Subprocess::new("git")
            .args(vec!["checkout", &self.git_ref])
            .current_dir(&cache_dir)
            .run_silent()?;

        copy_dir(cache_dir.join(&self.name), &filter_dir)?;

        let filter_config_path = filter_dir.join("filter.json");
        let mut filter_config = read_json::<Value>(&filter_config_path)?;
        filter_config["version"] = ref_to_version(&self.git_ref).into();
        write_json(&filter_config_path, &filter_config)?;

        let filter_config = FilterRemote::new(&self.name)?;
        for entry in filter_config.filters {
            let filter = entry.to_filter(&self.name)?;
            filter.install_dependencies(filter_dir.to_owned())?;
        }
        Ok(())
    }
}

fn get_git_ref(name: &str, url: &str, version: Option<String>) -> RglResult<String> {
    if let Some(version) = &version {
        if let Ok(version) = Version::parse(&version) {
            return Ok(format!("{name}-{version}"));
        }
    }
    if version.is_none() || version == Some("latest".to_owned()) {
        let output = Subprocess::new("git")
            .args(["ls-remote", "--tags", &format!("https://{url}")])
            .run_silent()?;
        let output = String::from_utf8(output.stdout).unwrap();

        let tag = output
            .trim()
            .split("\n")
            .filter_map(|line| {
                if let Some(version) = line.split(&format!("refs/tags/{name}-")).last() {
                    if let Ok(version) = Version::parse(version) {
                        return Some(format!("{name}-{version}"));
                    }
                }
                None
            })
            .last();
        if let Some(tag) = tag {
            return Ok(tag);
        }
    }
    if version.is_none() || version == Some("HEAD".to_owned()) {
        let output = Subprocess::new("git")
            .args(["ls-remote", "--symref", &format!("https://{url}"), "HEAD"])
            .run_silent()?;
        let output = String::from_utf8(output.stdout).unwrap();

        let sha = match output.split("\n").nth(1) {
            Some(line) => line.split("\t").nth(0),
            None => None,
        };
        if let Some(sha) = sha {
            return Ok(sha.to_owned());
        }
    }
    Err(RglError::FilterVersionResolveFailed {
        name: name.to_owned(),
        url: name.to_owned(),
        version: match version {
            Some(version) => version.to_owned(),
            None => "latest".to_owned(),
        },
    })
}

pub fn ref_to_version(git_ref: &str) -> String {
    match Version::parse(git_ref.split("-").nth(1).unwrap_or("-")) {
        Ok(version) => version.to_string(),
        Err(_) => git_ref.to_owned(),
    }
}
