//! The collection functions of `core` (Language Reference, sections 3.7 and 4; the catalogue's
//! Collections group). `vars(name, ..., x1..x50)` (row 39) names variables outside a frame
//! method, where a bare list of names has no frame to be looked up in (section 6, "Names inside
//! a frame's methods"); a range among them stays unexpanded until the list meets a frame
//! (section 3.7, "Variable ranges").

use crate::runtime::{Args, CallContext, Fail, Function};
use crate::value::Value;
use zeff_registry::{FunctionSpec, ParamSpec, Rest};

fn vars(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let mut names: Vec<String> = Vec::new();
    let mut push = |value: &Value| match value {
        Value::Var(name) => names.push(name.clone()),
        Value::Vars(more) => names.extend(more.iter().cloned()),
        _ => {}
    };
    if let Some(first) = args.get(0) {
        push(first);
    }
    for value in &args.rest {
        push(value);
    }
    Ok(Value::Vars(names))
}

pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "vars",
            package: "core",
            method: false,
            params: vec![ParamSpec::required("name", "Var | Vars")],
            rest: Some(Rest::Values("Var | Vars")),
            returns: "Vars",
            diagnostics: vec![],
            help: include_str!("../help/vars.md"),
            accuracy: None,
        },
        body: vars,
    }]
}
