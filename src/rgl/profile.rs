use super::{RglError, RglResult, RunContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use simplelog::info;
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub export: Export,
    pub filters: Vec<FilterRunner>,
}

#[derive(Serialize, Deserialize)]
pub struct Export {
    pub target: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterRunner {
    Filter {
        #[serde(rename = "filter")]
        filter_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<HashMap<String, Value>>,
    },
    ProfileFilter {
        #[serde(rename = "profile")]
        profile_name: String,
    },
}

impl Profile {
    pub fn run(&self, context: &RunContext, temp: &PathBuf) -> RglResult<()> {
        for entry in self.filters.iter() {
            match entry {
                FilterRunner::Filter {
                    filter_name,
                    arguments,
                    settings,
                } => {
                    let filter_def = context.get_filter_def(filter_name)?;
                    let mut run_args: Vec<String> = vec![];
                    if let Some(settings) = settings {
                        run_args = vec![serde_json::to_string(settings).unwrap()]
                    }
                    if let Some(args) = arguments {
                        run_args.extend(args.iter().map(|x| x.to_owned()));
                    }

                    info!("Running filter <b>{filter_name}</>");
                    filter_def.to_filter(&filter_name)?.run(temp, &run_args)?;
                }
                FilterRunner::ProfileFilter { profile_name } => {
                    if profile_name == &context.root_profile {
                        return Err(RglError::CircularProfileReference {
                            profile_name: context.root_profile.to_owned(),
                        });
                    }
                    let profile = context.get_profile(profile_name)?;

                    info!("Running <b>{profile_name}</> nested profile");
                    profile.run(&context, temp)?;
                }
            }
        }
        Ok(())
    }
}
