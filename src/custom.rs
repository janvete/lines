use crate::app::CustomState;

pub fn build_commands(state: &CustomState) -> Vec<String> {
    let command = state.command.trim();
    let pre_command = state.pre_command.trim();

    if command.is_empty() && pre_command.is_empty() {
        return Vec::new();
    }

    let selected = state.selected_lines();
    if selected.is_empty() {
        return Vec::new();
    }

    let prefix = if pre_command.is_empty() {
        String::new()
    } else {
        format!("{} ", pre_command)
    };

    selected
        .iter()
        .map(|line| {
            let line = line.trim();
            let built = if command.is_empty() {
                line.to_string()
            } else if command.contains("{}") {
                escape_dollars(&command.replace("{}", line))
            } else {
                let escaped = escape_dollars(command).replace('"', "\\\"");
                format!("{} \"{}\"", line, escaped)
            };
            format!("{}{}", prefix, built)
        })
        .collect()
}

/// Escape `$` as `\$` unless it is already escaped, so that variables are
/// expanded by the remote shell rather than the local one when the built
/// command is passed to `ssh`.
fn escape_dollars(command: &str) -> String {
    let mut result = String::with_capacity(command.len());
    let mut prev_backslash = false;
    for c in command.chars() {
        if c == '$' && !prev_backslash {
            result.push('\\');
        }
        result.push(c);
        prev_backslash = c == '\\' && !prev_backslash;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{CustomLine, CustomState};

    fn make_state(lines: Vec<&str>, selected: Vec<bool>, command: &str, pre_command: &str) -> CustomState {
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(0));
        CustomState {
            lines: lines
                .into_iter()
                .map(|c| CustomLine {
                    section: "test".to_string(),
                    command: c.to_string(),
                })
                .collect(),
            selected,
            cursor: 0,
            focus: crate::app::CustomFocus::Lines,
            command: command.to_string(),
            pre_command: pre_command.to_string(),
            list_state,
        }
    }

    #[test]
    fn test_build_commands_append() {
        let state = make_state(
            vec!["ssh root@ip1", "ssh root@ip2", "ssh root@ip3"],
            vec![false, true, true],
            "lsblk",
            "",
        );
        assert_eq!(
            build_commands(&state),
            vec!["ssh root@ip2 \"lsblk\"", "ssh root@ip3 \"lsblk\""]
        );
    }

    #[test]
    fn test_build_commands_placeholder() {
        let state = make_state(vec!["root@ip1", "root@ip2"], vec![true, true], "ssh {} lsblk", "");
        assert_eq!(
            build_commands(&state),
            vec!["ssh root@ip1 lsblk", "ssh root@ip2 lsblk"]
        );
    }

    #[test]
    fn test_build_commands_empty_selection() {
        let state = make_state(vec!["ssh root@ip1"], vec![false], "lsblk", "");
        assert!(build_commands(&state).is_empty());
    }

    #[test]
    fn test_build_commands_pre_command() {
        let state = make_state(
            vec!["ssh root@ip1", "ssh root@ip2"],
            vec![true, true],
            "ping",
            "lview",
        );
        assert_eq!(
            build_commands(&state),
            vec![
                "lview ssh root@ip1 \"ping\"",
                "lview ssh root@ip2 \"ping\""
            ]
        );
    }

    #[test]
    fn test_build_commands_pre_command_placeholder() {
        let state = make_state(
            vec!["root@ip1", "root@ip2"],
            vec![true, true],
            "ssh {} lsblk",
            "lview",
        );
        assert_eq!(
            build_commands(&state),
            vec!["lview ssh root@ip1 lsblk", "lview ssh root@ip2 lsblk"]
        );
    }

    #[test]
    fn test_build_commands_pre_command_only() {
        let state = make_state(vec!["ssh root@ip1", "ssh root@ip2"], vec![false, true], "", "lview");
        assert_eq!(build_commands(&state), vec!["lview ssh root@ip2"]);
    }

    #[test]
    fn test_build_commands_escapes_dollar_variables() {
        let state = make_state(
            vec!["ssh root@ip1"],
            vec![true],
            "echo $HOME && echo $USER",
            "",
        );
        assert_eq!(
            build_commands(&state),
            vec!["ssh root@ip1 \"echo \\$HOME && echo \\$USER\""]
        );
    }

    #[test]
    fn test_build_commands_preserves_already_escaped_dollars() {
        let state = make_state(
            vec!["ssh root@ip1"],
            vec![true],
            "echo \\$HOME",
            "",
        );
        assert_eq!(
            build_commands(&state),
            vec!["ssh root@ip1 \"echo \\$HOME\""]
        );
    }

    #[test]
    fn test_build_commands_placeholder_escapes_dollars() {
        let state = make_state(vec!["root@ip1"], vec![true], "ssh {} echo $HOSTNAME", "");
        assert_eq!(
            build_commands(&state),
            vec!["ssh root@ip1 echo \\$HOSTNAME"]
        );
    }
}
