use super::{empty_dir, find_mojang_dir, move_dir, RglError, RglResult};
use simplelog::info;
use std::path::{Path, PathBuf};

fn get_export_paths(name: &str, target: &str) -> RglResult<(PathBuf, PathBuf)> {
    match target {
        "development" => {
            let mojang_dir = find_mojang_dir();
            let bp = mojang_dir
                .join("development_behavior_packs")
                .join(format!("{name}_bp"));
            let rp = mojang_dir
                .join("development_resource_packs")
                .join(format!("{name}_rp"));
            Ok((bp, rp))
        }
        "local" => {
            let build = Path::new(".").join("build");
            let bp = build.join("BP");
            let rp = build.join("RP");
            Ok((bp, rp))
        }
        _ => Err(RglError::ExportTargetError {
            target: target.to_owned(),
        }),
    }
}

pub fn export_project(name: &str, temp: &PathBuf, target: &str) -> RglResult<()> {
    let (bp, rp) = get_export_paths(name, target)?;

    if target == "local" {
        empty_dir(Path::new(".").join("build"))?;
    }

    info!(
        "Moving files to target location: \n\
        \tBP: {} \n\
        \tRP: {}",
        bp.display(),
        rp.display()
    );

    move_dir(temp.join("BP"), bp)?;
    move_dir(temp.join("RP"), rp)?;
    empty_dir(&temp)?;
    Ok(())
}
