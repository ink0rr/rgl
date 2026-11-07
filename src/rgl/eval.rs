use super::RunContext;
use crate::logger::Logger;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use oxide_eval::{context::ContextEntry, Evaluator};
use serde_json::{json, Value};
use std::env::consts::{ARCH, OS};
use std::path::Path;
use std::sync::LazyLock;

static ENV: LazyLock<serde_json::Map<String, Value>> = LazyLock::new(|| {
    std::env::vars_os()
        .filter_map(|(k, v)| Some((k.into_string().ok()?, Value::String(v.into_string().ok()?))))
        .collect()
});

pub struct Eval(Evaluator);

impl Eval {
    pub fn new(
        ctx: &RunContext,
        filter_location: &Path,
        settings: Option<&IndexMap<String, Value>>,
    ) -> Self {
        let context = [
            ("os", json!(OS)),
            ("arch", json!(ARCH)),
            // Keep this at 0.0.0 because some filters refuse to run
            // when the version is below a certain threshold.
            ("version", json!("0.0.0")),
            ("profile", json!(ctx.profile)),
            (
                "filterLocation",
                json!(filter_location.display().to_string()),
            ),
            ("settings", json!(settings)),
            ("debug", json!(Logger::get_debug())),
            ("pi", json!(std::f64::consts::PI)),
            (
                "project",
                json!({
                    "name": ctx.config.get_name(),
                    "author": ctx.config.get_author(),
                }),
            ),
            ("mode", json!(ctx.mode.to_string())),
            ("nested", json!(ctx.is_nested())),
            ("initial", json!(ctx.is_initial())),
            ("env", Value::Object(ENV.clone())),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), ContextEntry::Variable(v)))
        .collect();
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
