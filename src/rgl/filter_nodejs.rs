use super::{Filter, FilterContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterNodejs {
    pub script: String,
    /// Optional path to the folder with the `package.json` file. If not
    /// specified, the parent of the script path is used instead. Mirrors
    /// Go's `NodeJSFilterDefinition.Requirements`. Only used when the
    /// resolved runtime is `node` (Bun/Deno manage dependencies
    /// differently, matching Go's `BunFilterDefinition`/
    /// `DenoFilterDefinition`, which don't have a `requirements` field).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
}

/// The JS runtime family a `nodejs` filter is executed with.
///
/// Go picks the filter's actual type (bun/deno/node) at config-parse time,
/// based on the user config's `node_runner_override`. rgl keeps a single
/// `FilterNodejs` type and instead resolves the runtime at run time, so a
/// `nodejs`-declared filter can still be executed as a `bun`/`deno` script
/// when overridden.
enum NodeRuntime {
    Node,
    Bun,
    Deno,
}

impl NodeRuntime {
    /// Resolves the runtime for a filter named `filter_name`, consulting
    /// `node_runner_override` (filter-specific entry, then `"*"`). Absent an
    /// override, this always resolves to `Node`, which keeps existing
    /// projects (including ones setting the legacy `nodejs_runtime` field to
    /// something like `"bun"`) running exactly as before: `nodejs_runtime`
    /// only changes which binary is invoked in the `Node` branch below, not
    /// the invocation shape.
    fn resolve(filter_name: &str) -> Self {
        match UserConfig::node_runner_override(filter_name).as_deref() {
            Some("bun") => NodeRuntime::Bun,
            Some("deno") => NodeRuntime::Deno,
            _ => NodeRuntime::Node,
        }
    }
}

impl Filter for FilterNodejs {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = context.filter_dir.join(&self.script);
        let mut subprocess = match NodeRuntime::resolve(&context.name) {
            NodeRuntime::Node => {
                let mut subprocess = Subprocess::new(UserConfig::node_runner());
                subprocess.arg(&script);
                subprocess
            }
            NodeRuntime::Bun => {
                let mut subprocess = Subprocess::new(UserConfig::bun_runner());
                subprocess.arg("run").arg(&script);
                subprocess
            }
            NodeRuntime::Deno => {
                let mut subprocess = Subprocess::new(UserConfig::deno_runner());
                subprocess.args(["run", "-A", "--no-lock"]).arg(&script);
                subprocess
            }
        };
        subprocess
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir);
        if UserConfig::subprocess_logging() {
            subprocess.run_with_prefix(&context.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let filter_dir = match &self.requirements {
            Some(requirements) => context.filter_dir.join(requirements),
            None => context.filter_dir(&self.script),
        };
        match NodeRuntime::resolve(&context.name) {
            NodeRuntime::Node => {
                if filter_dir.join("package.json").exists() {
                    Subprocess::new(UserConfig::npm_runner())
                        .arg("i")
                        .current_dir(filter_dir)
                        .run()?;
                }
            }
            NodeRuntime::Bun => {
                Subprocess::new(UserConfig::bun_runner())
                    .arg("i")
                    .current_dir(filter_dir)
                    .run()?;
            }
            // Matches Go's `DenoFilterDefinition`, which has no
            // `InstallDependencies` step at all.
            NodeRuntime::Deno => {}
        }
        Ok(())
    }
}
