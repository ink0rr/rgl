use super::{FilterRunner, RglError, RglResult, RunContext};
use serde::{Deserialize, Serialize};
use simplelog::info;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub export: Export,
    pub filters: Vec<FilterRunner>,
}

#[derive(Serialize, Deserialize)]
pub struct Export {
    pub target: String,
}

impl Profile {
    pub fn run(&self, context: &RunContext, temp: &PathBuf) -> RglResult<()> {
        for entry in self.filters.iter() {
            if entry.profile == Some(context.root_profile.to_string().to_owned()) {
                return Err(RglError::CircularProfileReference {
                    profile_name: context.root_profile.to_owned(),
                });
            }

            if let Some(profile_name) = &entry.profile {
                let profile = &context.get_profile(profile_name)?;
                info!("Running <b>{profile_name}</> nested profile");
                profile.run(&context, temp)?;
                continue;
            }

            if let Some(filter_name) = &entry.filter {
                let filter_def = &context.get_filter_def(filter_name)?;
                info!("Running filter <b>{filter_name}</>");
                let mut run_args: Vec<String> = vec![];
                if let Some(settings) = &entry.settings {
                    run_args = vec![serde_json::to_string(settings).unwrap()]
                }
                if let Some(args) = &entry.arguments {
                    run_args.extend(args.iter().map(|x| x.to_owned()));
                }
                filter_def.to_filter(&filter_name)?.run(temp, &run_args)?;
                continue;
            }
        }
        Ok(())
    }
}
