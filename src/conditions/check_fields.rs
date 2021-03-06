use crate::{
    conditions::{Condition, ConditionConfig, ConditionDescription},
    event::Value,
    Event,
};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use string_cache::DefaultAtom as Atom;

#[derive(Deserialize, Serialize, Clone, Derivative)]
#[serde(untagged)]
#[derivative(Debug)]
pub enum CheckFieldsPredicateArg {
    #[derivative(Debug = "transparent")]
    String(String),
    #[derivative(Debug = "transparent")]
    Integer(i64),
    #[derivative(Debug = "transparent")]
    Float(f64),
    #[derivative(Debug = "transparent")]
    Boolean(bool),
}

pub trait CheckFieldsPredicate: std::fmt::Debug + Send + Sync {
    fn check(&self, e: &Event) -> bool;
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct EqualsPredicate {
    target: Atom,
    arg: CheckFieldsPredicateArg,
}

impl EqualsPredicate {
    pub fn new(
        target: String,
        arg: &CheckFieldsPredicateArg,
    ) -> Result<Box<dyn CheckFieldsPredicate>, String> {
        Ok(Box::new(Self {
            target: target.into(),
            arg: arg.clone(),
        }))
    }
}

impl CheckFieldsPredicate for EqualsPredicate {
    fn check(&self, event: &Event) -> bool {
        match event {
            Event::Log(l) => l.get(&self.target).map_or(false, |v| match &self.arg {
                CheckFieldsPredicateArg::String(s) => s.as_bytes() == v.as_bytes(),
                CheckFieldsPredicateArg::Integer(i) => match v {
                    Value::Integer(vi) => *i == *vi,
                    Value::Float(vf) => *i == *vf as i64,
                    _ => false,
                },
                CheckFieldsPredicateArg::Float(f) => match v {
                    Value::Float(vf) => *f == *vf,
                    Value::Integer(vi) => *f == *vi as f64,
                    _ => false,
                },
                CheckFieldsPredicateArg::Boolean(b) => match v {
                    Value::Boolean(vb) => *b == *vb,
                    _ => false,
                },
            }),
            Event::Metric(m) => m
                .tags
                .as_ref()
                .and_then(|t| t.get(self.target.as_ref()))
                .map_or(false, |v| match &self.arg {
                    CheckFieldsPredicateArg::String(s) => s.as_bytes() == v.as_bytes(),
                    _ => false,
                }),
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct ContainsPredicate {
    target: Atom,
    arg: String,
}

impl ContainsPredicate {
    pub fn new(
        target: String,
        arg: &CheckFieldsPredicateArg,
    ) -> Result<Box<dyn CheckFieldsPredicate>, String> {
        match arg {
            CheckFieldsPredicateArg::String(s) => Ok(Box::new(Self {
                target: target.into(),
                arg: s.clone(),
            })),
            _ => Err("contains predicate requires a string argument".to_owned()),
        }
    }
}

impl CheckFieldsPredicate for ContainsPredicate {
    fn check(&self, event: &Event) -> bool {
        match event {
            Event::Log(l) => l
                .get(&self.target)
                .map_or(false, |v| v.to_string_lossy().contains(&self.arg)),
            _ => false,
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct StartsWithPredicate {
    target: Atom,
    arg: String,
}

impl StartsWithPredicate {
    pub fn new(
        target: String,
        arg: &CheckFieldsPredicateArg,
    ) -> Result<Box<dyn CheckFieldsPredicate>, String> {
        match arg {
            CheckFieldsPredicateArg::String(s) => Ok(Box::new(Self {
                target: target.into(),
                arg: s.clone(),
            })),
            _ => Err("starts_with predicate requires a string argument".to_owned()),
        }
    }
}

impl CheckFieldsPredicate for StartsWithPredicate {
    fn check(&self, event: &Event) -> bool {
        match event {
            Event::Log(l) => l
                .get(&self.target)
                .map_or(false, |v| v.to_string_lossy().starts_with(&self.arg)),
            _ => false,
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct EndsWithPredicate {
    target: Atom,
    arg: String,
}

impl EndsWithPredicate {
    pub fn new(
        target: String,
        arg: &CheckFieldsPredicateArg,
    ) -> Result<Box<dyn CheckFieldsPredicate>, String> {
        match arg {
            CheckFieldsPredicateArg::String(s) => Ok(Box::new(Self {
                target: target.into(),
                arg: s.clone(),
            })),
            _ => Err("ends_with predicate requires a string argument".to_owned()),
        }
    }
}

impl CheckFieldsPredicate for EndsWithPredicate {
    fn check(&self, event: &Event) -> bool {
        match event {
            Event::Log(l) => l
                .get(&self.target)
                .map_or(false, |v| v.to_string_lossy().ends_with(&self.arg)),
            _ => false,
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct NotEqualsPredicate {
    target: Atom,
    arg: String,
}

impl NotEqualsPredicate {
    pub fn new(
        target: String,
        arg: &CheckFieldsPredicateArg,
    ) -> Result<Box<dyn CheckFieldsPredicate>, String> {
        Ok(Box::new(Self {
            target: target.into(),
            arg: match arg {
                CheckFieldsPredicateArg::String(s) => s.clone(),
                CheckFieldsPredicateArg::Integer(a) => format!("{}", a),
                CheckFieldsPredicateArg::Float(a) => format!("{}", a),
                CheckFieldsPredicateArg::Boolean(a) => format!("{}", a),
            },
        }))
    }
}

impl CheckFieldsPredicate for NotEqualsPredicate {
    fn check(&self, event: &Event) -> bool {
        match event {
            Event::Log(l) => l
                .get(&self.target)
                .map(|f| f.as_bytes())
                .map_or(false, |b| b != self.arg.as_bytes()),
            Event::Metric(m) => m
                .tags
                .as_ref()
                .and_then(|t| t.get(self.target.as_ref()))
                .map_or(false, |v| v.as_bytes() != self.arg.as_bytes()),
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct RegexPredicate {
    target: Atom,
    regex: Regex,
}

impl RegexPredicate {
    pub fn new(
        target: String,
        arg: &CheckFieldsPredicateArg,
    ) -> Result<Box<dyn CheckFieldsPredicate>, String> {
        let pattern = match arg {
            CheckFieldsPredicateArg::String(s) => s.clone(),
            _ => return Err("regex predicate requires a string argument".to_owned()),
        };
        let regex = Regex::new(&pattern)
            .map_err(|error| format!("Invalid regex \"{}\": {}", pattern, error))?;
        let target = target.into();
        Ok(Box::new(Self { target, regex }))
    }
}

impl CheckFieldsPredicate for RegexPredicate {
    fn check(&self, event: &Event) -> bool {
        match event {
            Event::Log(log) => log
                .get(&self.target)
                .map(|field| field.to_string_lossy())
                .map_or(false, |field| self.regex.is_match(&field)),
            Event::Metric(metric) => metric
                .tags
                .as_ref()
                .and_then(|tags| tags.get(self.target.as_ref()))
                .map_or(false, |field| self.regex.is_match(field)),
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct ExistsPredicate {
    target: Atom,
    arg: bool,
}

impl ExistsPredicate {
    pub fn new(
        target: String,
        arg: &CheckFieldsPredicateArg,
    ) -> Result<Box<dyn CheckFieldsPredicate>, String> {
        match arg {
            CheckFieldsPredicateArg::Boolean(b) => Ok(Box::new(Self {
                target: target.into(),
                arg: *b,
            })),
            _ => Err("exists predicate requires a boolean argument".to_owned()),
        }
    }
}

impl CheckFieldsPredicate for ExistsPredicate {
    fn check(&self, event: &Event) -> bool {
        (match event {
            Event::Log(l) => l.get(&self.target).is_some(),
            Event::Metric(m) => m
                .tags
                .as_ref()
                .map_or(false, |t| t.contains_key(self.target.as_ref())),
        }) == self.arg
    }
}

//------------------------------------------------------------------------------

fn build_predicate(
    predicate: &str,
    target: String,
    arg: &CheckFieldsPredicateArg,
) -> Result<Box<dyn CheckFieldsPredicate>, String> {
    match predicate {
        "eq" | "equals" => EqualsPredicate::new(target, arg),
        "neq" | "not_equals" => NotEqualsPredicate::new(target, arg),
        "contains" => ContainsPredicate::new(target, arg),
        "prefix" => {
            warn!(
                message = "The \"prefix\" comparison predicate is deprecated, use \"starts_with\" instead",
                %target,
            );
            StartsWithPredicate::new(target, arg)
        }
        "starts_with" => StartsWithPredicate::new(target, arg),
        "ends_with" => EndsWithPredicate::new(target, arg),
        "exists" => ExistsPredicate::new(target, arg),
        "regex" => RegexPredicate::new(target, arg),
        _ => Err(format!("predicate type '{}' not recognized", predicate)),
    }
}

fn build_predicates(
    map: &IndexMap<String, CheckFieldsPredicateArg>,
) -> Result<IndexMap<String, Box<dyn CheckFieldsPredicate>>, Vec<String>> {
    let mut predicates: IndexMap<String, Box<dyn CheckFieldsPredicate>> = IndexMap::new();
    let mut errors = Vec::new();

    for (target_pred, arg) in map {
        if target_pred
            .rfind('.')
            .and_then(|i| {
                if i > 0 && i < target_pred.len() - 1 {
                    Some(i)
                } else {
                    None
                }
            })
            .and_then(|i| {
                let mut target = target_pred.clone();
                let pred = target.split_off(i + 1);
                target.truncate(target.len() - 1);
                match build_predicate(&pred, target, arg) {
                    Ok(pred) => {
                        predicates.insert(format!("{}: {:?}", target_pred, arg), pred);
                    }
                    Err(err) => errors.push(err),
                }
                Some(())
            })
            .is_none()
        {
            errors.push(format!("predicate not found in check_fields value '{}', format must be <target>.<predicate>", target_pred));
        }
    }

    if errors.is_empty() {
        Ok(predicates)
    } else {
        Err(errors)
    }
}

//------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CheckFieldsConfig {
    #[serde(flatten, default)]
    predicates: IndexMap<String, CheckFieldsPredicateArg>,
}

inventory::submit! {
    ConditionDescription::new::<CheckFieldsConfig>("check_fields")
}

#[typetag::serde(name = "check_fields")]
impl ConditionConfig for CheckFieldsConfig {
    fn build(&self) -> crate::Result<Box<dyn Condition>> {
        build_predicates(&self.predicates)
            .map(|preds| -> Box<dyn Condition> { Box::new(CheckFields { predicates: preds }) })
            .map_err(|errs| {
                if errs.len() > 1 {
                    let mut err_fmt = errs.join("\n");
                    err_fmt.insert_str(0, "failed to parse predicates:\n");
                    err_fmt
                } else {
                    errs[0].clone()
                }
                .into()
            })
    }
}

//------------------------------------------------------------------------------

pub struct CheckFields {
    predicates: IndexMap<String, Box<dyn CheckFieldsPredicate>>,
}

impl Condition for CheckFields {
    fn check(&self, e: &Event) -> bool {
        self.predicates.iter().find(|(_, p)| !p.check(e)).is_none()
    }

    fn check_with_context(&self, e: &Event) -> Result<(), String> {
        let failed_preds = self
            .predicates
            .iter()
            .filter(|(_, p)| !p.check(e))
            .map(|(n, _)| n.to_owned())
            .collect::<Vec<_>>();
        if failed_preds.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "predicates failed: [ {} ]",
                failed_preds.join(", ")
            ))
        }
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::Event;

    #[test]
    fn check_predicate_errors() {
        let cases = vec![
            ("foo", "predicate not found in check_fields value 'foo', format must be <target>.<predicate>"),
            (".nah", "predicate not found in check_fields value '.nah', format must be <target>.<predicate>"),
            ("", "predicate not found in check_fields value '', format must be <target>.<predicate>"),
            ("what.", "predicate not found in check_fields value 'what.', format must be <target>.<predicate>"),
            ("foo.not_real", "predicate type 'not_real' not recognized"),
        ];

        let mut aggregated_preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        let mut exp_errs = Vec::new();
        for (pred, exp) in cases {
            aggregated_preds.insert(pred.into(), CheckFieldsPredicateArg::String("foo".into()));
            exp_errs.push(exp);

            let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
            preds.insert(pred.into(), CheckFieldsPredicateArg::String("foo".into()));

            assert_eq!(
                CheckFieldsConfig { predicates: preds }
                    .build()
                    .err()
                    .unwrap()
                    .to_string(),
                exp.to_owned()
            );
        }

        let mut exp_err = exp_errs.join("\n");
        exp_err.insert_str(0, "failed to parse predicates:\n");

        assert_eq!(
            CheckFieldsConfig {
                predicates: aggregated_preds
            }
            .build()
            .err()
            .unwrap()
            .to_string(),
            exp_err
        );
    }

    #[test]
    fn check_field_equals() {
        let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        preds.insert(
            "message.equals".into(),
            CheckFieldsPredicateArg::String("foo".into()),
        );
        preds.insert(
            "other_thing.eq".into(),
            CheckFieldsPredicateArg::String("bar".into()),
        );

        let cond = CheckFieldsConfig { predicates: preds }.build().unwrap();

        let mut event = Event::from("neither");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(
                "predicates failed: [ message.equals: \"foo\", other_thing.eq: \"bar\" ]"
                    .to_owned()
            )
        );

        event.as_mut_log().insert("message", "foo");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ other_thing.eq: \"bar\" ]".to_owned())
        );

        event.as_mut_log().insert("other_thing", "bar");
        assert_eq!(cond.check(&event), true);
        assert_eq!(cond.check_with_context(&event), Ok(()));

        event.as_mut_log().insert("message", "not foo");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ message.equals: \"foo\" ]".to_owned())
        );
    }

    #[test]
    fn check_field_contains() {
        let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        preds.insert(
            "message.contains".into(),
            CheckFieldsPredicateArg::String("foo".into()),
        );
        preds.insert(
            "other_thing.contains".into(),
            CheckFieldsPredicateArg::String("bar".into()),
        );

        let cond = CheckFieldsConfig { predicates: preds }.build().unwrap();

        let mut event = Event::from("neither");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(
                "predicates failed: [ message.contains: \"foo\", other_thing.contains: \"bar\" ]"
                    .to_owned()
            )
        );

        event.as_mut_log().insert("message", "hello foo world");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ other_thing.contains: \"bar\" ]".to_owned())
        );

        event.as_mut_log().insert("other_thing", "hello bar world");
        assert_eq!(cond.check(&event), true);
        assert_eq!(cond.check_with_context(&event), Ok(()));

        event.as_mut_log().insert("message", "not fo0");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ message.contains: \"foo\" ]".to_owned())
        );
    }

    #[test]
    fn check_field_prefix() {
        let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        preds.insert(
            "message.prefix".into(),
            CheckFieldsPredicateArg::String("foo".into()),
        );
        preds.insert(
            "other_thing.prefix".into(),
            CheckFieldsPredicateArg::String("bar".into()),
        );

        let cond = CheckFieldsConfig { predicates: preds }.build().unwrap();

        let mut event = Event::from("neither");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(
                "predicates failed: [ message.prefix: \"foo\", other_thing.prefix: \"bar\" ]"
                    .to_owned()
            )
        );

        event.as_mut_log().insert("message", "foo hello world");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ other_thing.prefix: \"bar\" ]".to_owned())
        );

        event.as_mut_log().insert("other_thing", "bar hello world");
        assert_eq!(cond.check(&event), true);
        assert_eq!(cond.check_with_context(&event), Ok(()));

        event.as_mut_log().insert("message", "not prefixed");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ message.prefix: \"foo\" ]".to_owned())
        );
    }

    #[test]
    fn check_field_starts_with() {
        let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        preds.insert(
            "message.starts_with".into(),
            CheckFieldsPredicateArg::String("foo".into()),
        );
        preds.insert(
            "other_thing.starts_with".into(),
            CheckFieldsPredicateArg::String("bar".into()),
        );

        let cond = CheckFieldsConfig { predicates: preds }.build().unwrap();

        let mut event = Event::from("neither");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(
                "predicates failed: [ message.starts_with: \"foo\", other_thing.starts_with: \"bar\" ]"
                    .to_owned()
            )
        );

        event.as_mut_log().insert("message", "foo hello world");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ other_thing.starts_with: \"bar\" ]".to_owned())
        );

        event.as_mut_log().insert("other_thing", "bar hello world");
        assert_eq!(cond.check(&event), true);
        assert_eq!(cond.check_with_context(&event), Ok(()));

        event.as_mut_log().insert("message", "not prefixed");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ message.starts_with: \"foo\" ]".to_owned())
        );
    }

    #[test]
    fn check_field_ends_with() {
        let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        preds.insert(
            "message.ends_with".into(),
            CheckFieldsPredicateArg::String("foo".into()),
        );
        preds.insert(
            "other_thing.ends_with".into(),
            CheckFieldsPredicateArg::String("bar".into()),
        );

        let cond = CheckFieldsConfig { predicates: preds }.build().unwrap();

        let mut event = Event::from("neither");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(
                "predicates failed: [ message.ends_with: \"foo\", other_thing.ends_with: \"bar\" ]"
                    .to_owned()
            )
        );

        event.as_mut_log().insert("message", "hello world foo");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ other_thing.ends_with: \"bar\" ]".to_owned())
        );

        event.as_mut_log().insert("other_thing", "hello world bar");
        assert_eq!(cond.check(&event), true);
        assert_eq!(cond.check_with_context(&event), Ok(()));

        event.as_mut_log().insert("message", "not suffixed");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ message.ends_with: \"foo\" ]".to_owned())
        );
    }

    #[test]
    fn check_field_not_equals() {
        let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        preds.insert(
            "message.not_equals".into(),
            CheckFieldsPredicateArg::String("foo".into()),
        );
        preds.insert(
            "other_thing.neq".into(),
            CheckFieldsPredicateArg::String("bar".into()),
        );

        let cond = CheckFieldsConfig { predicates: preds }.build().unwrap();

        let mut event = Event::from("not foo");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ other_thing.neq: \"bar\" ]".to_owned())
        );

        event.as_mut_log().insert("other_thing", "not bar");
        assert_eq!(cond.check(&event), true);
        assert_eq!(cond.check_with_context(&event), Ok(()));

        event.as_mut_log().insert("other_thing", "bar");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ other_thing.neq: \"bar\" ]".to_owned())
        );

        event.as_mut_log().insert("message", "foo");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(
                "predicates failed: [ message.not_equals: \"foo\", other_thing.neq: \"bar\" ]"
                    .to_owned()
            )
        );
    }

    #[test]
    fn check_field_regex() {
        let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        preds.insert(
            "message.regex".into(),
            CheckFieldsPredicateArg::String("^start".into()),
        );
        preds.insert(
            "other_thing.regex".into(),
            CheckFieldsPredicateArg::String("end$".into()),
        );

        let cond = CheckFieldsConfig { predicates: preds }.build().unwrap();

        let mut event = Event::from("starts with a bang");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(r#"predicates failed: [ other_thing.regex: "end$" ]"#.to_owned())
        );

        event.as_mut_log().insert("other_thing", "at the end");
        assert_eq!(cond.check(&event), true);
        assert_eq!(cond.check_with_context(&event), Ok(()));

        event.as_mut_log().insert("other_thing", "end up here");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(r#"predicates failed: [ other_thing.regex: "end$" ]"#.to_owned())
        );

        event.as_mut_log().insert("message", "foo");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err(
                r#"predicates failed: [ message.regex: "^start", other_thing.regex: "end$" ]"#
                    .to_owned()
            )
        );
    }

    #[test]
    fn check_field_exists() {
        let mut preds: IndexMap<String, CheckFieldsPredicateArg> = IndexMap::new();
        preds.insert("foo.exists".into(), CheckFieldsPredicateArg::Boolean(true));
        preds.insert("bar.exists".into(), CheckFieldsPredicateArg::Boolean(false));

        let cond = CheckFieldsConfig { predicates: preds }.build().unwrap();

        let mut event = Event::from("ignored field");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ foo.exists: true ]".to_owned())
        );

        event.as_mut_log().insert("foo", "not ignored");
        assert_eq!(cond.check(&event), true);
        assert_eq!(cond.check_with_context(&event), Ok(()));

        event.as_mut_log().insert("bar", "also not ignored");
        assert_eq!(cond.check(&event), false);
        assert_eq!(
            cond.check_with_context(&event),
            Err("predicates failed: [ bar.exists: false ]".to_owned())
        );
    }
}
