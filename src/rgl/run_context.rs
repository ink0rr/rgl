use super::{Config, Temp};
use anyhow::{bail, Result};
use strum::Display;

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "lowercase")]
pub enum RunMode {
    Run,
    Watch { initial: bool },
}

pub struct RunContext<'a> {
    pub config: &'a Config,
    pub temp: &'a Temp,
    pub profile: &'a str,
    pub extra_args: &'a [String],
    pub mode: RunMode,
    parent: Option<&'a RunContext<'a>>,
}

impl<'a> RunContext<'a> {
    pub fn new(
        config: &'a Config,
        temp: &'a Temp,
        profile: &'a str,
        extra_args: &'a [String],
        mode: RunMode,
    ) -> Self {
        Self {
            config,
            temp,
            profile,
            extra_args,
            mode,
            parent: None,
        }
    }

    /// Context for running `profile` as a nested profile of this one.
    /// Extra CLI arguments are not forwarded into nested profiles.
    pub fn nested_profile<'b>(&'b self, profile: &'b str) -> Result<RunContext<'b>> {
        let mut current = Some(self);
        while let Some(ctx) = current {
            if ctx.profile == profile {
                bail!("Found circular profile reference in <profile>{profile}</>");
            }
            current = ctx.parent;
        }
        Ok(RunContext {
            profile,
            extra_args: &[],
            parent: Some(self),
            ..*self
        })
    }

    pub fn is_nested(&self) -> bool {
        self.parent.is_some()
    }

    pub fn is_initial(&self) -> bool {
        !matches!(self.mode, RunMode::Watch { initial: false })
    }
}
