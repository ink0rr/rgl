use super::{
    Filter, FilterBun, FilterDeno, FilterInfo, NodeRunner, RunContext, Subprocess, UserConfig,
};
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilterNodejs {
    pub script: String,
}

impl FilterNodejs {
    fn as_bun(&self) -> FilterBun {
        FilterBun {
            script: self.script.clone(),
        }
    }

    fn as_deno(&self) -> FilterDeno {
        FilterDeno {
            script: self.script.clone(),
        }
    }
}

impl Filter for FilterNodejs {
    fn run(&self, ctx: &RunContext, info: &FilterInfo, run_args: &[String]) -> Result<()> {
        match UserConfig::node_runner_override(&info.name) {
            NodeRunner::Bun => return self.as_bun().run(ctx, info, run_args),
            NodeRunner::Deno => return self.as_deno().run(ctx, info, run_args),
            NodeRunner::Nodejs => {}
        }
        let script = info.filter_dir.join(&self.script);
        let mut subprocess = Subprocess::new(UserConfig::node_runner());
        subprocess
            .arg(script)
            .args(ctx.settings_arg()?)
            .args(run_args)
            .current_dir(&ctx.temp.root)
            .setup_env(&info.filter_dir);
        if UserConfig::subprocess_logging() {
            subprocess.run_with_prefix(&info.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self, info: &FilterInfo) -> Result<()> {
        match UserConfig::node_runner_override(&info.name) {
            NodeRunner::Bun => return self.as_bun().install_dependencies(info),
            NodeRunner::Deno => return self.as_deno().install_dependencies(info),
            NodeRunner::Nodejs => {}
        }
        let filter_dir = info.filter_dir(&self.script);
        if filter_dir.join("package.json").exists() {
            Subprocess::new(UserConfig::npm_runner())
                .arg("i")
                .current_dir(filter_dir)
                .run()?;
        }
        Ok(())
    }
}
