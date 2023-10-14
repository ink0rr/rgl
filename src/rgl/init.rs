use super::{empty_dir, write_file, write_json, Config, Manifest, PackType, RglError, RglResult};
use dialoguer::{theme::ColorfulTheme, Input};
use log::info;
use semver::Version;
use serde_json::json;
use std::env;
use uuid::Uuid;

pub fn init() -> RglResult<()> {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => return Err(RglError::Wrap(e.into()).into()),
    };

    if cwd
        .read_dir()
        .map(|mut i| i.next().is_some())
        .unwrap_or(true)
    {
        return Err(RglError::CurrentDirNotEmpty);
    }

    let name = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Project name")
        .default(cwd.file_name().unwrap().to_string_lossy().to_string())
        .interact_text()
        .unwrap();

    let min_engine_version = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Minimum engine version")
        .default("1.20.30".to_owned())
        .validate_with(|input: &String| -> Result<(), String> {
            if Version::parse(&input).is_ok() {
                Ok(())
            } else {
                Err("Invalid version".to_string())
            }
        })
        .interact_text()
        .unwrap();

    let bp = "./packs/BP";
    let rp = "./packs/RP";

    let bp_header = Uuid::new_v4().to_string();
    let rp_header = Uuid::new_v4().to_string();

    empty_dir(format!("{bp}/texts"))?;
    empty_dir(format!("{rp}/texts"))?;
    empty_dir("./data")?;

    write_json(
        format!("{bp}/manifest.json"),
        &Manifest::new(
            PackType::Behavior,
            &bp_header,
            &rp_header,
            &min_engine_version,
        ),
    )?;
    write_json(
        format!("{rp}/manifest.json"),
        &Manifest::new(
            PackType::Resource,
            &rp_header,
            &bp_header,
            &min_engine_version,
        ),
    )?;

    write_json(format!("{bp}/texts/languages.json"), &json!(["en_US"]))?;
    write_json(format!("{rp}/texts/languages.json"), &json!(["en_US"]))?;

    write_file(
        format!("{bp}/texts/en_US.lang"),
        create_lang(PackType::Behavior, &name),
    )?;
    write_file(
        format!("{rp}/texts/en_US.lang"),
        create_lang(PackType::Resource, &name),
    )?;
    write_file(".gitignore", "/build\n/.regolith\n")?;

    Config::new(name).save()?;
    info!("Project initialized");
    Ok(())
}

fn create_lang(pack_type: PackType, name: &str) -> String {
    let (pack_type, suffix) = match pack_type {
        PackType::Behavior => ("Behavior Pack", "BP"),
        PackType::Resource => ("Resource Pack", "RP"),
    };
    format!(
        "pack.name={name} {suffix}\n\
         pack.description={pack_type} for {name}\n"
    )
}
