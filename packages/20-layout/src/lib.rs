#![allow(clippy::missing_panics_doc)]

use extism_pdk::*;
use serde::Deserialize;
use serde_json::{json, Value};

const USAGE: &str = "usage: maw layout <name> [--to <session:window>]";
const PRESETS: &[&str] = &[
    "even-horizontal",
    "even-vertical",
    "main-horizontal",
    "main-vertical",
    "tiled",
];

#[link(wasm_import_module = "extism:host/user")]
extern "C" {
    #[link_name = "maw.tmux.command"]
    fn maw_tmux_command(input: u64) -> u64;
}

#[derive(Default, Deserialize)]
struct Context {
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct LayoutPlan {
    preset: String,
    target: Option<String>,
    tmux_args: Vec<String>,
}

#[plugin_fn]
pub unsafe fn handle(input: String) -> FnResult<String> {
    let context = serde_json::from_str::<Context>(&input).unwrap_or_default();
    Ok(match run(&context.args) {
        Ok(output) => json!({"ok": true, "output": output}),
        Err(error) => json!({"ok": false, "error": error}),
    }
    .to_string())
}

fn run(args: &[String]) -> Result<String, String> {
    let plan = build_plan(args)?;
    let response = host_call(
        maw_tmux_command,
        json!({"command": "select-layout", "args": plan.tmux_args}).to_string(),
    );
    if response.get("ok").and_then(Value::as_bool) != Some(true) {
        let error = response
            .get("error")
            .or_else(|| response.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("unknown host error");
        return Err(format!("layout: select-layout failed: {error}"));
    }
    Ok(format!(
        "layout {} applied to {}\n",
        plan.preset,
        plan.target.as_deref().unwrap_or("current window")
    ))
}

fn build_plan(args: &[String]) -> Result<LayoutPlan, String> {
    let mut preset = None;
    let mut target = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return Err(USAGE.to_owned()),
            "--to" => {
                target = Some(
                    args.get(index + 1)
                        .ok_or_else(|| "layout: --to requires a target".to_owned())?
                        .clone(),
                );
                index += 1;
            }
            value if value.starts_with("--to=") => target = Some(value[5..].to_owned()),
            value if value.starts_with('-') => {
                return Err(format!("layout: unknown argument {value}"));
            }
            value if preset.is_some() => {
                let _ = value;
                return Err("layout: expected exactly one layout name".to_owned());
            }
            value => preset = Some(value.to_owned()),
        }
        index += 1;
    }
    let preset = preset.ok_or_else(|| USAGE.to_owned())?;
    validate_preset(&preset)?;
    let target = target
        .map(|target| {
            validate_target(&target)?;
            Ok::<_, String>(window_target(&target))
        })
        .transpose()?;
    let mut tmux_args = Vec::new();
    if let Some(target) = &target {
        tmux_args.extend(["-t".to_owned(), target.clone()]);
    }
    tmux_args.push(preset.clone());
    Ok(LayoutPlan {
        preset,
        target,
        tmux_args,
    })
}

fn validate_preset(value: &str) -> Result<(), String> {
    if PRESETS.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "layout: invalid layout '{value}'. Valid: {}",
            PRESETS.join(", ")
        ))
    }
}

fn validate_target(value: &str) -> Result<(), String> {
    if value.is_empty() || value.trim() != value || value == "--" || value.starts_with('-') {
        return Err(
            "layout: target must be non-empty, unpadded, not '--', and not start with '-'"
                .to_owned(),
        );
    }
    if value.chars().any(char::is_control) {
        return Err("layout: target must not contain control characters".to_owned());
    }
    if !value.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':' | '.' | '/' | '@' | '%')
    }) {
        return Err("layout: target contains unsupported characters".to_owned());
    }
    Ok(())
}

fn window_target(target: &str) -> String {
    let Some((head, tail)) = target.rsplit_once('.') else {
        return target.to_owned();
    };
    if !head.is_empty() && !tail.is_empty() && tail.bytes().all(|byte| byte.is_ascii_digit()) {
        head.to_owned()
    } else {
        target.to_owned()
    }
}

fn host_call(function: unsafe extern "C" fn(u64) -> u64, input: String) -> Value {
    let Ok(memory) = Memory::from_bytes(input.as_bytes()) else {
        return Value::Null;
    };
    let offset = memory.offset();
    let output = unsafe { function(offset) };
    memory.free();
    Memory::find(output)
        .and_then(|memory| {
            let bytes = memory.to_vec();
            memory.free();
            serde_json::from_slice(&bytes).ok()
        })
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn builds_current_window_and_targeted_plans() {
        let current = build_plan(&strings(&["main-vertical"])).expect("current layout");
        assert_eq!(current.target, None);
        assert_eq!(current.tmux_args, strings(&["main-vertical"]));

        let targeted =
            build_plan(&strings(&["tiled", "--to", "team:work.2"])).expect("targeted layout");
        assert_eq!(targeted.target.as_deref(), Some("team:work"));
        assert_eq!(targeted.tmux_args, strings(&["-t", "team:work", "tiled"]));
    }

    #[test]
    fn rejects_invalid_preset_and_target_before_host_call() {
        assert!(build_plan(&strings(&["broken"]))
            .unwrap_err()
            .contains("invalid layout"));
        assert!(build_plan(&strings(&["tiled", "--to", "bad;target"]))
            .unwrap_err()
            .contains("unsupported"));
    }
}
