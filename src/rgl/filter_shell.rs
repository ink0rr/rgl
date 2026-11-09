use super::{Filter, FilterInfo, RunContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FilterShell {
    pub command: String,
}

impl Filter for FilterShell {
    fn run(&self, ctx: &RunContext, info: &FilterInfo, run_args: &[String]) -> Result<()> {
        let shell = if cfg!(windows) { "powershell" } else { "sh" };
        let settings = ctx.settings_arg()?;
        let args = settings
            .as_deref()
            .into_iter()
            .chain(run_args.iter().map(String::as_str));
        let command = format!("{} {}", self.command, shlex::try_join(args)?);
        let mut subprocess = Subprocess::new(shell);
        subprocess
            .arg("-c")
            .arg(command)
            .current_dir(&ctx.temp.root)
            .setup_env(&info.filter_dir);
        if UserConfig::subprocess_logging() {
            subprocess.run_with_prefix(&info.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }
}
