#![allow(clippy::missing_panics_doc)]
use extism_pdk::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fmt::Write as _;

#[link(wasm_import_module = "extism:host/user")]
extern "C" {
    #[link_name = "maw.cli.run"]
    fn maw_cli_run(input: u64) -> u64;
    #[link_name = "maw.tmux.list_sessions"]
    fn maw_tmux_list_sessions(input: u64) -> u64;
    #[link_name = "maw.tmux.send_keys"]
    fn maw_tmux_send_keys(input: u64) -> u64;
}

const USAGE: &str = "usage: maw incubate <source-repo> [--stem <name>] [--from <oracle>] [--root] [--seed] [--org <org>] [--note <text>] [--nickname <pretty>] [--fast] [--split] [--dry-run] [--flash | --contribute] [--no-trigger] [--trigger <text>]";

#[derive(Deserialize)]
struct Context {
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
struct Options {
    source: String,
    stem: Option<String>,
    trigger: Option<String>,
    no_trigger: bool,
    flash: bool,
    contribute: bool,
    from: Option<String>,
    from_repo: Option<String>,
    org: Option<String>,
    issue: Option<u64>,
    note: Option<String>,
    nickname: Option<String>,
    fast: bool,
    root: bool,
    blank: bool,
    seed: bool,
    split: bool,
    dry_run: bool,
    signal_on_birth: bool,
    force: bool,
    track_vault: bool,
    sync_peers: bool,
}

fn host_call(function: unsafe extern "C" fn(u64) -> u64, input: String) -> Value {
    let Ok(memory) = Memory::from_bytes(input.as_bytes()) else {
        return Value::Null;
    };
    let offset = memory.offset();
    let output = unsafe { function(offset) };
    memory.free();
    Memory::find(output).map_or(Value::Null, |memory| {
        let bytes = memory.to_vec();
        memory.free();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    })
}

fn result(ok: bool, output: &str, error: &str) -> String {
    if ok {
        json!({"ok": true, "output": output}).to_string()
    } else {
        json!({"ok": false, "error": error}).to_string()
    }
}

#[plugin_fn]
pub unsafe fn handle(input: String) -> FnResult<String> {
    let context: Context = serde_json::from_str(&input).unwrap_or(Context { args: Vec::new() });
    match run(&context.args) {
        Ok(output) => Ok(result(true, &output, "")),
        Err(error) => Ok(result(false, "", &error)),
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let options = parse_args(args)?;
    if options.flash && options.contribute {
        return Err("--flash and --contribute are mutually exclusive".to_owned());
    }
    let stem = options
        .stem
        .clone()
        .unwrap_or_else(|| derive_stem(&options.source));
    validate_target(&stem, "stem")?;
    let trigger = if options.no_trigger {
        None
    } else {
        Some(build_trigger(&options)?)
    };
    let mut output = run_bud(&stem, &options)?;
    if options.dry_run {
        if let Some(trigger) = trigger {
            let _ = writeln!(
                output,
                "  \x1b[36m⬡\x1b[0m [dry-run] would send \x1b[33m{trigger}\x1b[0m to {stem}"
            );
        } else {
            output
                .push_str("  \x1b[36m⬡\x1b[0m [dry-run] --no-trigger: would NOT fire /incubate\n");
        }
        return Ok(output);
    }
    let Some(trigger) = trigger else {
        output.push_str("  \x1b[90m○\x1b[0m --no-trigger: bud + wake done, skipping /incubate\n");
        return Ok(output);
    };
    let Some(target) = resolve_target(&stem) else {
        let _ = writeln!(
            output,
            "  \x1b[33m⚠\x1b[0m could not resolve {stem} after wake — skipping {trigger}"
        );
        let _ = writeln!(
            output,
            "  \x1b[90m  try manually: maw send-text {stem} '{trigger}'\x1b[0m"
        );
        return Ok(output);
    };
    let _ = writeln!(
        output,
        "  \x1b[36m🔔\x1b[0m firing \x1b[33m{trigger}\x1b[0m → {stem}"
    );
    let sent = host_call(
        maw_tmux_send_keys,
        json!({
            "target": target,
            "keys": [trigger],
            "literal": true,
            "enter": false,
            "allowAiPane": true
        })
        .to_string(),
    );
    if sent.get("ok").and_then(Value::as_bool) == Some(true) {
        output.push_str("  \x1b[32m✓\x1b[0m incubation dispatched\n");
    } else {
        let error = host_error(&sent);
        let _ = writeln!(output, "  \x1b[33m⚠\x1b[0m send-text failed: {error}");
        let _ = writeln!(
            output,
            "  \x1b[90m  try manually: maw send-text {stem} '{trigger}'\x1b[0m"
        );
    }
    Ok(output)
}

fn run_bud(stem: &str, options: &Options) -> Result<String, String> {
    let response = host_call(
        maw_cli_run,
        json!({"command": "bud", "args": bud_args(stem, options)?}).to_string(),
    );
    if response.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!("incubate: bud failed: {}", host_error(&response)));
    }
    let value = response.get("value").unwrap_or(&response);
    let status = value.get("status").and_then(Value::as_i64).unwrap_or(-1);
    if status != 0 {
        let error = value
            .get("stderr")
            .and_then(Value::as_str)
            .unwrap_or("bud failed")
            .trim_end();
        return Err(format!("incubate: bud failed: {error}"));
    }
    Ok(value
        .get("stdout")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned())
}

fn resolve_target(stem: &str) -> Option<String> {
    let response = host_call(maw_tmux_list_sessions, "{}".to_owned());
    let value = response.get("value").unwrap_or(&response);
    let sessions = value.get("sessions")?.as_array()?;
    let session = sessions.iter().find(|session| {
        session
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name.eq_ignore_ascii_case(stem))
    })?;
    let name = session.get("name")?.as_str()?;
    let windows = session.get("windows")?.as_array()?;
    let window = windows
        .iter()
        .find(|window| window.get("active").and_then(Value::as_bool) == Some(true))
        .or_else(|| windows.first())?;
    let index = window.get("index")?.as_u64()?;
    Some(format!("{name}:{index}"))
}

fn host_error(value: &Value) -> String {
    value
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("host call failed")
        .to_owned()
}

fn bud_args(stem: &str, options: &Options) -> Result<Vec<String>, String> {
    validate_target(stem, "stem")?;
    let mut args = vec![stem.to_owned()];
    push_option(&mut args, "--from", options.from.as_deref())?;
    push_option(&mut args, "--from-repo", options.from_repo.as_deref())?;
    push_option(&mut args, "--org", options.org.as_deref())?;
    if let Some(issue) = options.issue {
        let issue = u32::try_from(issue)
            .map_err(|_| format!("incubate: --issue value {issue} is too large for bud"))?;
        args.push(format!("--issue={issue}"));
    }
    push_opaque_option(&mut args, "--note", options.note.as_deref());
    push_opaque_option(&mut args, "--nickname", options.nickname.as_deref());
    for (flag, enabled) in [
        ("--fast", options.fast),
        ("--root", options.root),
        ("--blank", options.blank),
        ("--seed", options.seed),
        ("--split", options.split),
        ("--dry-run", options.dry_run),
        ("--signal-on-birth", options.signal_on_birth),
        ("--force", options.force),
        ("--track-vault", options.track_vault),
        ("--sync-peers", options.sync_peers),
    ] {
        if enabled {
            args.push(flag.to_owned());
        }
    }
    Ok(args)
}

fn push_option(args: &mut Vec<String>, flag: &str, value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        validate_path(value, flag)?;
        args.push(format!("{flag}={value}"));
    }
    Ok(())
}

fn push_opaque_option(args: &mut Vec<String>, flag: &str, value: Option<&str>) {
    if let Some(value) = value {
        args.push(format!("{flag}={value}"));
    }
}

fn build_trigger(options: &Options) -> Result<String, String> {
    if let Some(trigger) = &options.trigger {
        validate_path(trigger, "trigger")?;
        return Ok(trigger.clone());
    }
    let mut command = format!("/incubate {}", options.source);
    if options.flash {
        command.push_str(" --flash");
    } else if options.contribute {
        command.push_str(" --contribute");
    }
    Ok(command)
}

fn derive_stem(source: &str) -> String {
    source
        .rsplit('/')
        .next()
        .unwrap_or(source)
        .strip_suffix(".git")
        .unwrap_or_else(|| source.rsplit('/').next().unwrap_or(source))
        .to_owned()
}

#[allow(clippy::too_many_lines)]
fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut positionals = Vec::new();
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        match arg.as_str() {
            "--help" | "-h" => return Err(USAGE.to_owned()),
            "--no-trigger" => options.no_trigger = true,
            "--flash" => options.flash = true,
            "--contribute" => options.contribute = true,
            "--fast" => options.fast = true,
            "--root" => options.root = true,
            "--blank" => options.blank = true,
            "--seed" => options.seed = true,
            "--split" => options.split = true,
            "--dry-run" => options.dry_run = true,
            "--signal-on-birth" => options.signal_on_birth = true,
            "--force" => options.force = true,
            "--track-vault" => options.track_vault = true,
            "--sync-peers" => options.sync_peers = true,
            flag @ ("--stem" | "--trigger" | "--from" | "--from-repo" | "--org" | "--issue"
            | "--note" | "--nickname") => {
                let value = required_value(args, index, flag)?;
                assign_value(&mut options, flag, value)?;
                index += 1;
            }
            value if value.starts_with("--") && value.contains('=') => {
                let (flag, value) = value.split_once('=').unwrap_or_default();
                assign_value(&mut options, flag, value.to_owned())?;
            }
            value if value.starts_with('-') => {
                return Err(format!("incubate: unknown argument {value}"));
            }
            value => positionals.push(value.to_owned()),
        }
        index += 1;
    }
    if positionals.len() != 1 {
        return Err(USAGE.to_owned());
    }
    options.source = positionals.remove(0);
    validate_path(&options.source, "source repo")?;
    if let Some(stem) = &options.stem {
        validate_target(stem, "stem")?;
    }
    validate_optional(options.from.as_deref(), "from")?;
    validate_optional(options.from_repo.as_deref(), "from-repo")?;
    validate_optional(options.org.as_deref(), "org")?;
    validate_optional(options.trigger.as_deref(), "trigger")?;
    Ok(options)
}

fn required_value(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    let value = args
        .get(index + 1)
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| format!("incubate: {flag} requires a value"))?;
    Ok(value.clone())
}

fn assign_value(options: &mut Options, flag: &str, value: String) -> Result<(), String> {
    match flag {
        "--stem" => options.stem = Some(value),
        "--trigger" => options.trigger = Some(value),
        "--from" => options.from = Some(value),
        "--from-repo" => options.from_repo = Some(value),
        "--org" => options.org = Some(value),
        "--issue" => {
            let issue = value
                .parse::<u64>()
                .ok()
                .filter(|issue| *issue > 0)
                .ok_or_else(|| format!("incubate: invalid --issue value {value}"))?;
            options.issue = Some(issue);
        }
        "--note" => options.note = Some(value),
        "--nickname" => options.nickname = Some(value),
        _ => return Err(format!("incubate: unknown argument {flag}")),
    }
    Ok(())
}

fn validate_optional(value: Option<&str>, label: &str) -> Result<(), String> {
    value.map_or(Ok(()), |value| validate_path(value, label))
}

fn validate_path(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.trim() != value
        || value.starts_with('-')
        || value.chars().any(char::is_control)
    {
        return Err(format!("incubate: {label} must be non-empty, unpadded, not start with '-', and contain no control characters"));
    }
    Ok(())
}

fn validate_target(value: &str, label: &str) -> Result<(), String> {
    validate_path(value, label)?;
    if value.chars().any(char::is_whitespace) {
        return Err(format!("incubate: {label} must not contain whitespace"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn parser_and_bud_arguments_preserve_flags() {
        let options = parse_args(&strings(&[
            "org/source",
            "--stem",
            "custom",
            "--from",
            "nova",
            "--org",
            "org",
            "--issue",
            "133",
            "--note=hello",
            "--nickname=Pretty",
            "--fast",
            "--root",
            "--blank",
            "--seed",
            "--split",
            "--dry-run",
            "--signal-on-birth",
            "--force",
            "--track-vault",
            "--sync-peers",
            "--flash",
        ]))
        .expect("parse");
        assert_eq!(
            bud_args("custom", &options).expect("bud args"),
            strings(&[
                "custom",
                "--from=nova",
                "--org=org",
                "--issue=133",
                "--note=hello",
                "--nickname=Pretty",
                "--fast",
                "--root",
                "--blank",
                "--seed",
                "--split",
                "--dry-run",
                "--signal-on-birth",
                "--force",
                "--track-vault",
                "--sync-peers",
            ])
        );
        assert_eq!(
            build_trigger(&options).expect("trigger"),
            "/incubate org/source --flash"
        );
    }

    #[test]
    fn opaque_equals_values_do_not_become_flags() {
        let options = parse_args(&strings(&[
            "org/source",
            "--note=--split --from=mallory",
            "--nickname=-leading",
            "--dry-run",
        ]))
        .expect("parse");
        let args = bud_args("source", &options).expect("bud args");
        assert!(args.contains(&"--note=--split --from=mallory".to_owned()));
        assert!(args.contains(&"--nickname=-leading".to_owned()));
        assert!(!options.split);
    }

    #[test]
    fn parser_guards_modes_and_targets() {
        let both =
            parse_args(&strings(&["org/source", "--flash", "--contribute"])).expect("parse both");
        assert!(both.flash && both.contribute);
        assert!(parse_args(&strings(&["org/source", "--stem", "-bad"])).is_err());
        assert_eq!(derive_stem("https://github.com/org/source.git"), "source");
    }
}
