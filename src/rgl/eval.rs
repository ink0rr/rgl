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
        version: &str,
        profile: &str,
        filter_location: impl AsRef<Path>,
        settings: &Option<IndexMap<String, Value>>,
    ) -> Self {
        let mut context: HashMap<String, ContextEntry> = vec![
            ("os", OS.to_string()),
            ("arch", ARCH.to_string()),
            ("version", version.to_string()),
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
        add_math_func(&mut context);
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

fn add_number_constant(context: &mut HashMap<String, ContextEntry>, name: &str, value: f64) {
    context.insert(
        name.to_string(),
        ContextEntry::Function(Box::new(move |_| -> Value {
            Value::Number(serde_json::Number::from_f64(value).unwrap())
        })),
    );
}

fn add_math_func1(context: &mut HashMap<String, ContextEntry>, name: &str, func: fn(f64) -> f64) {
    let name = name.to_string();
    context.insert(
        name.clone(),
        ContextEntry::Function(Box::new(move |args| -> Value {
            let num = args[0]
                .as_f64()
                .ok_or_else(|| anyhow!("{} argument must be a number", name))
                .unwrap();
            Value::Number(serde_json::Number::from_f64(func(num)).unwrap())
        })),
    );
}

fn add_math_func2(
    context: &mut HashMap<String, ContextEntry>,
    name: &str,
    func: fn(f64, f64) -> f64,
) {
    let name = name.to_string();
    context.insert(
        name.clone(),
        ContextEntry::Function(Box::new(move |args| {
            let a = args[0]
                .as_f64()
                .ok_or_else(|| anyhow!("{} argument must be a number", name))
                .unwrap();
            let b = args[1]
                .as_f64()
                .ok_or_else(|| anyhow!("{} argument must be a number", name))
                .unwrap();
            Value::Number(serde_json::Number::from_f64(func(a, b)).unwrap())
        })),
    );
}

fn add_math_func(context: &mut HashMap<String, ContextEntry>) {
    add_number_constant(context, "pi", std::f64::consts::PI);
    let unary_funcs: [(&str, fn(f64) -> f64); 13] = [
        ("floor", f64::floor),
        ("ceil", f64::ceil),
        ("round", f64::round),
        ("sin", f64::sin),
        ("cos", f64::cos),
        ("tan", f64::tan),
        ("asin", f64::asin),
        ("acos", f64::acos),
        ("atan", f64::atan),
        ("sqrt", f64::sqrt),
        ("abs", f64::abs),
        ("clamp", |x: f64| x.clamp(0.0, 1.0)),
        ("bitwiseNot", |x: f64| !(f64::round(x) as i64) as f64),
    ];
    for &(name, func) in &unary_funcs {
        add_math_func1(context, name, func);
    }
    let f2: [(&str, fn(f64, f64) -> f64); 10] = [
        ("atan2", f64::atan2),
        ("min", f64::min),
        ("max", f64::max),
        ("mod", |x, y| x % y),
        ("pow", f64::powf),
        ("bitwiseAnd", |x, y| {
            (f64::round(x) as i64 & f64::round(y) as i64) as f64
        }),
        ("bitwiseOr", |x, y| {
            (f64::round(x) as i64 | f64::round(y) as i64) as f64
        }),
        ("bitwiseXor", |x, y| {
            (f64::round(x) as i64 ^ f64::round(y) as i64) as f64
        }),
        ("bitshiftLeft", |x, y| {
            ((f64::round(x) as i64) << (f64::round(y) as i64)) as f64
        }),
        ("bitshiftRight", |x, y| {
            (f64::round(x) as i64 >> f64::round(y) as i64) as f64
        }),
    ];
    for &(name, func) in &f2 {
        add_math_func2(context, name, func);
    }
}
