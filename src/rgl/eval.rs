use crate::logger::Logger;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use oxide_eval::{context::ContextEntry, Evaluator};
use serde_json::Value;
use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

/// The run-wide `mode` value exposed to expressions: `"run"` for `rgl run`
/// (and other one-shot commands like `apply`), `"watch"` for `rgl watch`.
/// Mirrors Go's `ctx.IsInWatchMode()`. Set once near the start of the
/// command, before any profile is run.
static RUN_MODE: OnceLock<String> = OnceLock::new();

/// Whether this is the first profile run of the process. Mirrors Go's
/// `RunContext.Initial`, which starts `true` and is set to `false` right
/// after the first `RunProfile` call in the watch loop (and stays `true` for
/// the whole duration of `rgl run`, which only ever runs once).
static RUN_INITIAL: AtomicBool = AtomicBool::new(true);

/// The current project's `name`/`author`, exposed as the `project` object.
/// Mirrors Go's `projectData` in `prepareScope`.
static PROJECT_INFO: Mutex<(String, Option<String>)> = Mutex::new((String::new(), None));

/// Sets the run mode ("run" or "watch") for the whole process. Call once,
/// before running any profile.
pub fn set_run_mode(mode: &str) {
    let _ = RUN_MODE.set(mode.to_string());
}

/// Marks whether the profile run about to happen is the first one of the
/// watch session (or the only one, for `rgl run`/`rgl apply`).
pub fn set_run_initial(initial: bool) {
    RUN_INITIAL.store(initial, Ordering::Relaxed);
}

/// The `initial` value for the top-level profile run (i.e. the one started
/// directly from the command, before descending into any nested `profile`
/// filter). Go always evaluates this the same way for the whole top-level
/// profile: it's whatever `RunContext.Initial` was set to before `RunProfile`
/// was called (see `set_run_initial`).
pub fn get_run_initial() -> bool {
    RUN_INITIAL.load(Ordering::Relaxed)
}

/// Sets the project's `name`/`author`, used to populate the `project`
/// object exposed to expressions.
pub fn set_project_info(name: &str, author: Option<&str>) {
    if let Ok(mut info) = PROJECT_INFO.lock() {
        *info = (name.to_string(), author.map(|s| s.to_string()));
    }
}

pub struct Eval(Evaluator);

impl Eval {
    /// Creates an evaluator for a `when` condition or name-template
    /// expression.
    ///
    /// `nested` indicates whether the filter being evaluated belongs to a
    /// profile that was reached through a nested `profile` filter, rather
    /// than the profile requested directly on the command line. Mirrors
    /// Go's `prepareScope`'s `nested: ctx.Parent != nil`.
    ///
    /// `initial` indicates whether this is the first profile run of the
    /// process. Mirrors Go's `RunContext.Initial`, which is only ever
    /// non-zero for the top-level (non-nested) run context: `ProfileFilter`
    /// builds a fresh `RunContext` for a nested profile without copying
    /// `Initial` over, so it always defaults to `false` for anything
    /// evaluated inside a nested profile, regardless of the top-level run's
    /// `Initial` value.
    pub fn new(
        profile: &str,
        filter_location: &Path,
        settings: Option<IndexMap<String, Value>>,
        nested: bool,
        initial: bool,
    ) -> Self {
        let mode = RUN_MODE.get().map(String::as_str).unwrap_or("run");
        let (project_name, project_author) = PROJECT_INFO
            .lock()
            .map(|info| info.clone())
            .unwrap_or_default();
        let project: Value = [
            ("name".to_string(), Value::String(project_name)),
            (
                "author".to_string(),
                project_author.map(Value::String).unwrap_or(Value::Null),
            ),
        ]
        .into_iter()
        .collect();
        let env: Value = std::env::vars()
            .map(|(k, v)| (k, Value::String(v)))
            .collect();

        let mut context: HashMap<String, ContextEntry> = vec![
            ("os", OS.to_string()),
            ("arch", ARCH.to_string()),
            ("version", env!("CARGO_PKG_VERSION").to_string()),
            ("profile", profile.to_string()),
            ("filterLocation", filter_location.display().to_string()),
            ("mode", mode.to_string()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), ContextEntry::Variable(v.into())))
        .collect();
        if let Some(settings) = settings {
            context.insert(
                "settings".to_string(),
                ContextEntry::Variable(settings.into_iter().collect()),
            );
        } else {
            context.insert("settings".to_string(), ContextEntry::Variable(Value::Null));
        }
        context.insert(
            "debug".to_string(),
            ContextEntry::Variable(Logger::get_debug().into()),
        );
        context.insert(
            "pi".to_string(),
            ContextEntry::Variable(std::f64::consts::PI.into()),
        );
        context.insert("nested".to_string(), ContextEntry::Variable(nested.into()));
        context.insert(
            "initial".to_string(),
            ContextEntry::Variable(initial.into()),
        );
        context.insert("project".to_string(), ContextEntry::Variable(project));
        context.insert("env".to_string(), ContextEntry::Variable(env));
        Self(Evaluator::new(context))
    }

    pub fn bool(&self, expression: &str) -> Result<bool> {
        match self.0.evaluate(expression)? {
            Value::String(v) => Ok(!v.is_empty()),
            Value::Number(v) => Ok(v.as_f64().unwrap_or_default() != 0.0),
            Value::Bool(v) => Ok(v),
            Value::Null => Ok(false),
            value => Err(anyhow!("Invalid expression result: {:?}", value)),
        }
    }

    pub fn string(&self, expression: &str) -> Result<String> {
        match self.0.evaluate(expression)? {
            Value::String(s) => Ok(s),
            value => Err(anyhow!(
                "Expression evaluated to non-string value: {:?}",
                value
            )),
        }
    }
}
