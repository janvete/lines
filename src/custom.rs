use crate::app::CustomState;

pub fn build_commands(state: &CustomState) -> Vec<String> {
    let command = state.command.trim();
    if command.is_empty() {
        return Vec::new();
    }

    let selected = state.selected_lines();
    if selected.is_empty() {
        return Vec::new();
    }

    let pre_command = state.pre_command.trim();
    let prefix = if pre_command.is_empty() {
        String::new()
    } else {
        format!("{} ", pre_command)
    };

    selected
        .iter()
        .map(|line| {
            let line = line.trim();
            let built = if command.contains("{}") {
                command.replace("{}", line)
            } else {
                format!("{} \"{}\"", line, command.replace('"', "\\\""))
            };
            format!("{}{}", prefix, built)
        })
        .collect()
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
}
