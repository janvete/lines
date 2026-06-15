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

    selected
        .iter()
        .map(|line| {
            let line = line.trim();
            if command.contains("{}") {
                command.replace("{}", line)
            } else {
                format!("{} {}", line, command)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{CustomLine, CustomState};

    fn make_state(lines: Vec<&str>, selected: Vec<bool>, command: &str) -> CustomState {
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
        }
    }

    #[test]
    fn test_build_commands_append() {
        let state = make_state(
            vec!["ssh root@ip1", "ssh root@ip2", "ssh root@ip3"],
            vec![false, true, true],
            "lsblk",
        );
        assert_eq!(
            build_commands(&state),
            vec!["ssh root@ip2 lsblk", "ssh root@ip3 lsblk"]
        );
    }

    #[test]
    fn test_build_commands_placeholder() {
        let state = make_state(vec!["root@ip1", "root@ip2"], vec![true, true], "ssh {} lsblk");
        assert_eq!(
            build_commands(&state),
            vec!["ssh root@ip1 lsblk", "ssh root@ip2 lsblk"]
        );
    }

    #[test]
    fn test_build_commands_empty_selection() {
        let state = make_state(vec!["ssh root@ip1"], vec![false], "lsblk");
        assert!(build_commands(&state).is_empty());
    }
}
