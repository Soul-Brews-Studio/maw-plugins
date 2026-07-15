//! Agent-pane predicate — pure Rust, no extism host bindings.
//!
//! Mirrors maw-rs native `is_claude_like_pane` / `is_three_part_numeric_version`
//! (crates/maw-tmux/src/core_impl/action_resolution_parts/safety_layout_validation.rs):
//! Claude Code panes now report a bare version string (e.g. "2.1.207") as
//! `pane_current_command`, so name matching alone is a fleet-wide no-op
//! (maw-rs#520).

/// True when a pane's `pane_current_command` looks like an agent process.
#[must_use]
pub fn is_agent(command: &str) -> bool {
    let c = command.to_lowercase();
    if ["claude", "codex", "node", "thclaws"]
        .iter()
        .any(|v| c.contains(v))
    {
        return true;
    }
    is_three_part_numeric_version(c.trim())
}

/// Exactly three '.'-separated parts, each non-empty and all ASCII digits.
fn is_three_part_numeric_version(value: &str) -> bool {
    let mut parts = value.split('.');
    let first = parts.next().unwrap_or_default();
    let Some(second) = parts.next() else {
        return false;
    };
    let Some(third) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    [first, second, third]
        .iter()
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::is_agent;

    #[test]
    fn matches_agent_process_names() {
        for command in ["claude", "Claude", "codex", "node", "thclaws", "some-claude-wrapper"] {
            assert!(is_agent(command), "{command} should match");
        }
    }

    #[test]
    fn matches_bare_three_part_numeric_version() {
        assert!(is_agent("2.1.207"));
        assert!(is_agent("0.0.0"));
        assert!(is_agent("10.20.30"));
    }

    #[test]
    fn matches_version_with_surrounding_whitespace() {
        // native trims before the version check
        assert!(is_agent(" 2.1.207 "));
        assert!(is_agent("2.1.207\n"));
    }

    #[test]
    fn rejects_non_version_shapes() {
        assert!(!is_agent("2.1")); // two parts
        assert!(!is_agent("2.1.207.1")); // four parts
        assert!(!is_agent("2.1.beta")); // non-digit part
        assert!(!is_agent("v2.1.207")); // prefixed
        assert!(!is_agent("2..207")); // empty middle part
        assert!(!is_agent(""));
        assert!(!is_agent("2.1.207 extra")); // internal space survives trim
    }

    #[test]
    fn rejects_plain_shells_and_tools() {
        for command in ["zsh", "bash", "vim", "htop", "ssh", "python3.11"] {
            assert!(!is_agent(command), "{command} should not match");
        }
    }
}
