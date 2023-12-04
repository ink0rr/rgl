use crate::fs::{empty_dir, write_file, write_json};
use crate::info;
use crate::rgl::{Config, Manifest, PackType};
use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Input};
use semver::Version;
use serde_json::json;
use std::env;
use uuid::Uuid;

pub fn init(force: bool) -> Result<()> {
    let cwd = env::current_dir()?;
    let cwd_entries = cwd
        .read_dir()
        .context("Failed to read current directory")?
        .filter_map(|entry| {
            let file_name = entry.ok()?.file_name();
            let is_hidden = file_name.to_str()?.starts_with('.');
            (!is_hidden).then_some(())
        })
        .count();

    if !force && cwd_entries > 0 {
        bail!("Current directory is not empty")
    }

    let dirname = cwd
        .file_name()
        .and_then(|s| s.to_str())
        .context("Failed to get current directory name")?;

    let name = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Project name")
        .default(dirname.to_owned())
        .interact_text()?;

    let min_engine_version = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Minimum engine version")
        .default("1.20.40".to_owned())
        .validate_with(|input: &String| -> Result<(), String> {
            if Version::parse(input).is_ok() {
                Ok(())
            } else {
                Err("Invalid version".to_string())
            }
        })
        .interact_text()?;

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
