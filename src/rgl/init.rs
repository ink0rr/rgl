use super::{empty_dir, write_file, write_json, RglError, RglResult};
use dialoguer::{theme::ColorfulTheme, Input};
use serde_json::{json, Value};
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
        .default("1.20.30".to_string())
        .validate_with(|input: &String| -> Result<(), String> {
            if input.parse::<semver::Version>().is_ok() {
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
        &create_manifest(
            PackType::Behavior,
            &bp_header,
            &rp_header,
            &min_engine_version,
        ),
    )?;
    write_json(
        format!("{rp}/manifest.json"),
        &create_manifest(
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

    write_json(
        "./config.json",
        &json!({
            "$schema": "https://raw.githubusercontent.com/Bedrock-OSS/regolith-schemas/main/config/v1.1.json",
            "author": "Your name",
            "name": name,
            "packs": {
                "behaviorPack": bp,
                "resourcePack": rp,
            },
            "regolith": {
                "dataPath": "./data",
                "filterDefinitions": {},
                "profiles": {
                    "default": {
                        "export": {
                            "target": "development",
                        },
                        "filters": [],
                    },
                    "build": {
                        "export": {
                            "target": "local",
                        },
                        "filters": [
                            {
                                "profile": "default",
                            },
                        ],
                    },
                },
            },
        }),
    )
}

enum PackType {
    Behavior,
    Resource,
}

fn create_manifest(
    pack_type: PackType,
    header_uuid: &str,
    deps_uuid: &str,
    min_engine_version: &str,
) -> Value {
    let version = vec![1, 0, 0];
    json!({
        "format_version": 2,
        "header": {
            "name": "pack.name",
            "description": "pack.description",
            "uuid": header_uuid,
            "version": version,
            "min_engine_version": min_engine_version.split(".").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>(),
        },
        "modules": [
            {
                "type": match pack_type {
                    PackType::Behavior => "data",
                    PackType::Resource => "resources",
                },
                "uuid": Uuid::new_v4().to_string(),
                "version": version,
            },
        ],
        "dependencies": [
            {
                "uuid": deps_uuid,
                "version": version,
            },
        ],
    })
}

fn create_lang(pack_type: PackType, name: &str) -> String {
    let suffix = match pack_type {
        PackType::Behavior => "BP",
        PackType::Resource => "RP",
    };
    let pack_type = match pack_type {
        PackType::Behavior => "Behavior Pack",
        PackType::Resource => "Resource Pack",
    };
    format!(
        "pack.name={name} {suffix}\n\
         pack.description={pack_type} for {name}\n"
    )
}
