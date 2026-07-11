#![allow(clippy::missing_panics_doc)]
use extism_pdk::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const HEARTBEAT_MS: u64 = 30_000;
const RECONNECT_BASE_MS: u64 = 1_000;
const RECONNECT_MAX_MS: u64 = 60_000;
const USAGE: &str = "usage: maw-rs hub validate-workspace [--id <id>] [--hub-url <ws-url>] [--token <token>] [--shared-agent <agent>]... [--plan-json]\n       maw-rs hub load-workspaces --config-dir <dir> [--plan-json]\n       maw-rs hub constants [--plan-json]";

#[link(wasm_import_module = "extism:host/user")]
extern "C" {
    #[link_name = "maw.fs.mkdir"]
    fn maw_fs_mkdir(input: u64) -> u64;
    #[link_name = "maw.fs.list"]
    fn maw_fs_list(input: u64) -> u64;
    #[link_name = "maw.fs.read"]
    fn maw_fs_read(input: u64) -> u64;
}

fn call(function: unsafe extern "C" fn(u64) -> u64, input: Value) -> Value {
    let Ok(memory) = Memory::from_bytes(input.to_string().as_bytes()) else {
        return Value::Null;
    };
    let output = unsafe { function(memory.offset()) };
    memory.free();
    Memory::find(output)
        .and_then(|memory| {
            let bytes = memory.to_vec();
            memory.free();
            serde_json::from_slice(&bytes).ok()
        })
        .unwrap_or(Value::Null)
}
fn host_value(value: &Value) -> Result<&Value, String> {
    if value.get("ok").and_then(Value::as_bool) == Some(false) {
        return Err(value
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("host call failed")
            .to_owned());
    }
    Ok(value.get("value").unwrap_or(value))
}
fn result(output: Result<String, String>) -> String {
    match output {
        Ok(output) => json!({"ok":true,"output":output}),
        Err(error) => json!({"ok":false,"error":error}),
    }
    .to_string()
}

#[derive(Deserialize)]
struct Context {
    #[serde(default)]
    args: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceConfig {
    id: String,
    hub_url: String,
    token: String,
    shared_agents: Vec<String>,
}
enum Action {
    Constants(bool),
    Validate {
        json: bool,
        id: String,
        url: String,
        token: String,
        agents: Vec<String>,
    },
    Load {
        json: bool,
        dir: String,
    },
}

#[plugin_fn]
pub unsafe fn handle(input: String) -> FnResult<String> {
    let context: Context = serde_json::from_str(&input).unwrap_or(Context { args: Vec::new() });
    Ok(result(parse(&context.args).and_then(run)))
}

fn parse(args: &[String]) -> Result<Action, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage("hub: expected validate-workspace or load-workspaces"));
    };
    match command {
        "constants" => {
            let mut plan = false;
            for arg in &args[1..] {
                if arg == "--plan-json" {
                    plan = true
                } else {
                    return Err(format!("hub constants: unknown arg {arg}\nusage: maw-rs hub constants [--plan-json]\n"));
                }
            }
            Ok(Action::Constants(plan))
        }
        "validate-workspace" => parse_validate(&args[1..]),
        "load-workspaces" => parse_load(&args[1..]),
        other => Err(usage(&format!("hub: unknown subcommand {other}"))),
    }
}
fn value(args: &[String], index: usize, name: &str) -> Result<String, String> {
    args.get(index + 1)
        .cloned()
        .ok_or_else(|| usage(&format!("hub: missing {name} value")))
}
fn parse_validate(args: &[String]) -> Result<Action, String> {
    let (mut plan, mut id, mut url, mut token, mut agents) = (
        false,
        String::new(),
        String::new(),
        String::new(),
        Vec::new(),
    );
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--plan-json" => plan = true,
            "--id" => {
                id = value(args, i, "--id")?;
                i += 1;
            }
            "--hub-url" => {
                url = value(args, i, "--hub-url")?;
                i += 1;
            }
            "--token" => {
                token = value(args, i, "--token")?;
                i += 1;
            }
            "--shared-agent" => {
                agents.push(value(args, i, "--shared-agent")?);
                i += 1;
            }
            other => {
                return Err(usage(&format!(
                    "hub validate-workspace: unknown argument {other}"
                )))
            }
        }
        i += 1;
    }
    Ok(Action::Validate {
        json: plan,
        id,
        url,
        token,
        agents,
    })
}
fn parse_load(args: &[String]) -> Result<Action, String> {
    let (mut plan, mut dir) = (false, None);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--plan-json" => plan = true,
            "--config-dir" => {
                dir = Some(value(args, i, "--config-dir")?);
                i += 1;
            }
            other => {
                return Err(usage(&format!(
                    "hub load-workspaces: unknown argument {other}"
                )))
            }
        }
        i += 1;
    }
    Ok(Action::Load {
        json: plan,
        dir: dir.ok_or_else(|| usage("hub load-workspaces: --config-dir is required"))?,
    })
}
fn usage(message: &str) -> String {
    format!("{message}\n{USAGE}\n")
}

fn run(action: Action) -> Result<String, String> {
    match action {
        Action::Constants(plan) => Ok(if plan {
            constants_json()
        } else {
            format!("hub constants heartbeat-ms={HEARTBEAT_MS} reconnect-base-ms={RECONNECT_BASE_MS} reconnect-max-ms={RECONNECT_MAX_MS}\n")
        }),
        Action::Validate {
            json,
            id,
            url,
            token,
            agents,
        } => {
            let raw = json!({"id":id,"hubUrl":url,"token":token,"sharedAgents":agents});
            let reason = validate(&raw);
            Ok(if json {
                format!("{{\"command\":\"hub\",\"kind\":\"validate-workspace\",\"input\":{raw},\"ok\":{},\"reason\":{}}}\n", reason.is_none(), reason.as_deref().map_or("null".to_owned(), |s| json!(s).to_string()))
            } else {
                reason.map_or_else(|| "ok\n".to_owned(), |r| format!("invalid: {r}\n"))
            })
        }
        Action::Load { json, dir } => load(&dir)
            .map(|(configs, warnings)| {
                if json {
                    render_load(&configs, &warnings)
                } else {
                    format!("configs={} warnings={}\n", configs.len(), warnings.len())
                }
            })
            .map_err(|error| format!("hub load-workspaces: {error}")),
    }
}
fn validate(raw: &Value) -> Option<String> {
    let Some(object) = raw.as_object() else {
        return Some("not an object".to_owned());
    };
    for (field, reason) in [
        ("id", "missing/empty id"),
        ("hubUrl", "missing/empty hubUrl"),
        ("token", "missing/empty token"),
    ] {
        if object
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Some(reason.to_owned());
        }
    }
    if !object.get("sharedAgents").is_some_and(Value::is_array) {
        return Some("sharedAgents must be array".to_owned());
    }
    let url = object["hubUrl"].as_str().unwrap_or_default();
    let Some((protocol, rest)) = url.split_once("://") else {
        return Some("hubUrl not a valid URL".to_owned());
    };
    if protocol.is_empty() || rest.is_empty() || rest.chars().any(char::is_whitespace) {
        return Some("hubUrl not a valid URL".to_owned());
    }
    if matches!(protocol, "ws" | "wss") {
        None
    } else {
        Some(format!("hubUrl must be ws:|wss: (got {protocol}:)"))
    }
}

fn load(dir: &str) -> Result<(Vec<WorkspaceConfig>, Vec<String>), String> {
    let workspaces = Path::new(dir).join("workspaces");
    host_value(&call(maw_fs_mkdir, json!({"path":workspaces})))?;
    let mut files = Vec::new();
    let mut offset = 0;
    loop {
        let response = call(
            maw_fs_list,
            json!({"path":workspaces,"includeDirs":false,"maxEntries":1000,"offset":offset}),
        );
        let value = host_value(&response)?;
        files.extend(
            value["entries"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry["path"].as_str())
                .filter(|path| path.ends_with(".json"))
                .map(ToOwned::to_owned),
        );
        let Some(next) = value.get("nextOffset").and_then(Value::as_u64) else {
            break;
        };
        offset = next;
    }
    files.sort();
    let mut configs = Vec::new();
    let mut warnings = Vec::new();
    for path in files {
        load_file(&path, &mut configs, &mut warnings)
    }
    Ok((configs, warnings))
}
fn load_file(path: &str, configs: &mut Vec<WorkspaceConfig>, warnings: &mut Vec<String>) {
    let name = PathBuf::from(path)
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("<unknown>")
        .to_owned();
    let response = call(maw_fs_read, json!({"path":path,"maxBytes":10485760}));
    let raw = host_value(&response)
        .and_then(|v| v["content"].as_str().ok_or("missing content".to_owned()))
        .and_then(|text| serde_json::from_str::<Value>(text).map_err(|e| e.to_string()));
    let Ok(raw) = raw else {
        warnings.push(format!(
            "[hub] failed to parse workspace config: {name} {}",
            raw.unwrap_err()
        ));
        return;
    };
    if let Some(reason) = validate(&raw) {
        warnings.push(format!("[hub] invalid workspace config: {name} ({reason})"));
        return;
    }
    match serde_json::from_value(raw) {
        Ok(config) => configs.push(config),
        Err(error) => warnings.push(format!(
            "[hub] failed to parse workspace config: {name} {error}"
        )),
    }
}
fn render_load(configs: &[WorkspaceConfig], warnings: &[String]) -> String {
    let configs = configs
        .iter()
        .map(|c| {
            format!(
                "{{\"id\":{},\"hubUrl\":{},\"token\":{},\"sharedAgents\":{}}}",
                json!(c.id),
                json!(c.hub_url),
                json!(c.token),
                json!(c.shared_agents)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"command\":\"hub\",\"kind\":\"load-workspaces\",\"configs\":[{configs}],\"warnings\":{}}}\n", json!(warnings))
}
fn constants_json() -> String {
    format!(
        r#"{{"command":"hub","action":"constants","actions":["validate-workspace","load-workspaces"],"requiredFields":["id","hubUrl","token","sharedAgents"],"validProtocols":["ws","wss"],"workspaceDirName":"workspaces","fileExtension":"json","heartbeatMs":{HEARTBEAT_MS},"reconnectBaseMs":{RECONNECT_BASE_MS},"reconnectMaxMs":{RECONNECT_MAX_MS},"validationReasons":["not an object","missing/empty id","missing/empty hubUrl","missing/empty token","sharedAgents must be array","hubUrl must be ws:|wss: (got <protocol>:)","hubUrl not a valid URL"],"warningPrefixes":["[hub] failed to parse workspace config","[hub] invalid workspace config"]}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_reasons_match_native_contract() {
        assert_eq!(validate(&json!({})).as_deref(), Some("missing/empty id"));
        assert_eq!(
            validate(&json!({"id":"w","hubUrl":"http://hub","token":"t","sharedAgents":[]}))
                .as_deref(),
            Some("hubUrl must be ws:|wss: (got http:)")
        );
        assert!(
            validate(&json!({"id":"w","hubUrl":"wss://hub","token":"t","sharedAgents":[]}))
                .is_none()
        );
    }

    #[test]
    fn constants_and_usage_match_native_contract() {
        assert!(constants_json().contains("\"heartbeatMs\":30000"));
        assert_eq!(
            parse(&[]).err().expect("usage error"),
            usage("hub: expected validate-workspace or load-workspaces")
        );
    }
}
