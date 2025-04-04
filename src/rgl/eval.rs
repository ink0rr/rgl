use anyhow::{anyhow, Result};
use exprimo::{ContextEntry, Evaluator};
use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::path::Path;

pub struct FilterEvaluator {
    evaluator: Evaluator,
}

impl FilterEvaluator {
    pub fn new(
        profile: &str,
        filter_location: impl AsRef<Path>,
        settings: &Option<IndexMap<String, Value>>,
    ) -> Self {
        let mut context: HashMap<String, ContextEntry> = vec![
            ("os", OS.to_string()),
            ("arch", ARCH.to_string()),
            ("version", "0.0.0".to_string()),
            ("profile", profile.to_string()),
            (
                "filterLocation",
                filter_location.as_ref().to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), ContextEntry::Variable(Value::String(v))))
        .collect();
        if let Some(settings) = settings {
            context.insert("settings".to_string(), to_json(settings));
        }
        context.insert(
            "pi".to_string(),
            ContextEntry::Variable(Value::Number(
                serde_json::Number::from_f64(std::f64::consts::PI).unwrap(),
            )),
        );
        Self {
            evaluator: Evaluator::new(context),
        }
    }

    pub fn run(&self, expression: &str) -> Result<bool> {
        let result = self.evaluator.evaluate(expression)?;
        match result {
            Value::Bool(b) => Ok(b),
            Value::Null => Ok(false),
            Value::String(s) => Ok(!s.is_empty()),
            Value::Number(n) => Ok(n.as_f64().unwrap() != 0.0),
            _ => Err(anyhow!("Invalid expression result: {:?}", result)),
        }
    }
}

fn to_json(settings: &IndexMap<String, Value>) -> ContextEntry {
    let map = settings
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Map<String, Value>>();
    ContextEntry::Variable(Value::Object(map))
}
