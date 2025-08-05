use crate::logger::Logger;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use oxide_eval::{context::ContextEntry, Evaluator};
use serde_json::Value;
use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::path::Path;

pub struct Eval(Evaluator);

impl Eval {
    pub fn new(
        profile: &str,
        filter_location: &Path,
        settings: Option<IndexMap<String, Value>>,
    ) -> Self {
        let mut context: HashMap<String, ContextEntry> = vec![
            ("os", OS.to_string()),
            ("arch", ARCH.to_string()),
            ("version", "0.0.0".to_string()),
            ("profile", profile.to_string()),
            ("filterLocation", filter_location.display().to_string()),
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
