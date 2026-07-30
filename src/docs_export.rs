//! Clap command tree export for auto-generated docs (apps/docs).

use clap::{Arg, ArgAction, Command, CommandFactory};
use serde::Serialize;

use crate::cli::Cli;

#[derive(Debug, Clone, Serialize)]
pub struct CommandReferenceExport {
    pub generated_by: String,
    pub version: String,
    pub command_path: Vec<String>,
    pub command: CommandNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandNode {
    pub name: String,
    pub display_name: String,
    pub usage: String,
    pub about: Option<String>,
    pub long_about: Option<String>,
    pub after_help: Option<String>,
    pub aliases: Vec<String>,
    pub visible_aliases: Vec<String>,
    pub arguments: Vec<ArgumentNode>,
    pub subcommands: Vec<CommandNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArgumentNode {
    pub id: String,
    pub kind: String,
    pub display: String,
    pub short: Option<String>,
    pub long: Option<String>,
    pub help: Option<String>,
    pub long_help: Option<String>,
    pub required: bool,
    pub global: bool,
    pub repeatable: bool,
    pub positional_index: Option<usize>,
    pub default_values: Vec<String>,
    pub possible_values: Vec<String>,
}

pub fn export_command_reference(path: &[String]) -> Result<CommandReferenceExport, String> {
    let root = sanitize_command_tree(Cli::command());
    let command = select_command(root, path)?;

    Ok(CommandReferenceExport {
        generated_by: "weeping-angel-docs-export".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        command_path: path.to_vec(),
        command: export_command_node(&command, path),
    })
}

fn sanitize_command_tree(command: Command) -> Command {
    command
        .disable_help_flag(true)
        .mut_subcommands(sanitize_command_tree)
}

fn select_command(mut command: Command, path: &[String]) -> Result<Command, String> {
    for segment in path {
        let next = command
            .get_subcommands()
            .find(|candidate| command_matches(candidate, segment))
            .cloned()
            .ok_or_else(|| {
                let available = command
                    .get_subcommands()
                    .map(|candidate| candidate.get_name().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "Unknown command path segment `{segment}` under `{}`. Available: {}",
                    command.get_name(),
                    if available.is_empty() {
                        "<none>"
                    } else {
                        &available
                    }
                )
            })?;
        command = next;
    }
    Ok(command)
}

fn command_matches(command: &Command, segment: &str) -> bool {
    command.get_name() == segment
        || command.get_all_aliases().any(|alias| alias == segment)
        || command.get_visible_aliases().any(|alias| alias == segment)
}

fn export_command_node(command: &Command, path: &[String]) -> CommandNode {
    let display_name = if path.is_empty() {
        "weeping-angel".to_string()
    } else {
        format!("weeping-angel {}", path.join(" "))
    };
    let usage = {
        let mut usage_command = command.clone();
        usage_command
            .render_usage()
            .to_string()
            .replace(
                &format!("Usage: {}", command.get_name()),
                &format!("Usage: {display_name}"),
            )
    };

    CommandNode {
        name: command.get_name().to_string(),
        display_name,
        usage,
        about: command.get_about().map(|v| v.to_string()),
        long_about: command.get_long_about().map(|v| v.to_string()),
        after_help: command.get_after_help().map(|v| v.to_string()),
        aliases: command.get_all_aliases().map(str::to_string).collect(),
        visible_aliases: command.get_visible_aliases().map(str::to_string).collect(),
        arguments: command
            .get_arguments()
            .filter(|arg| !arg.is_hide_set())
            .map(export_argument_node)
            .collect(),
        subcommands: command
            .get_subcommands()
            .filter(|sc| !sc.is_hide_set())
            .map(|sc| {
                let mut next_path = path.to_vec();
                next_path.push(sc.get_name().to_string());
                export_command_node(sc, &next_path)
            })
            .collect(),
    }
}

fn export_argument_node(arg: &Arg) -> ArgumentNode {
    let default_values = arg
        .get_default_values()
        .iter()
        .map(|v| v.to_string_lossy().to_string())
        .collect();
    let possible_values = arg
        .get_possible_values()
        .into_iter()
        .map(|v| v.get_name().to_string())
        .collect();
    let short = arg.get_short().map(|v| format!("-{v}"));
    let long = arg.get_long().map(|v| format!("--{v}"));
    let positional_index = arg.get_index().map(|v| v as usize);
    let value_suffix = if argument_takes_value(arg) {
        format!(" {}", argument_value_placeholder(arg))
    } else {
        String::new()
    };

    let display = if arg.is_positional() {
        argument_value_placeholder(arg)
    } else if let Some(long) = long.as_deref() {
        format!("{long}{value_suffix}")
    } else if let Some(short) = short.as_deref() {
        format!("{short}{value_suffix}")
    } else {
        arg.get_id().as_str().to_string()
    };

    ArgumentNode {
        id: arg.get_id().as_str().to_string(),
        kind: if arg.is_positional() {
            "positional".into()
        } else {
            "option".into()
        },
        display,
        short,
        long,
        help: arg.get_help().map(|v| v.to_string()),
        long_help: arg.get_long_help().map(|v| v.to_string()),
        required: arg.is_required_set(),
        global: arg.is_global_set(),
        repeatable: matches!(arg.get_action(), ArgAction::Append | ArgAction::Count),
        positional_index,
        default_values,
        possible_values,
    }
}

fn argument_takes_value(arg: &Arg) -> bool {
    !matches!(
        arg.get_action(),
        ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count | ArgAction::Help | ArgAction::Version
    ) && !arg.is_positional()
        || arg.is_positional()
}

fn argument_value_placeholder(arg: &Arg) -> String {
    if let Some(names) = arg.get_value_names() {
        let rendered = names
            .iter()
            .map(|name| format!("<{}>", name.to_ascii_uppercase()))
            .collect::<Vec<_>>();
        if !rendered.is_empty() {
            return rendered.join(" ");
        }
    }
    format!(
        "<{}>",
        arg.get_id().as_str().replace('-', "_").to_ascii_uppercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_root_and_scan() {
        let root = export_command_reference(&[]).expect("root");
        assert_eq!(root.command.name, "weeping-angel");
        assert!(root.command.subcommands.iter().any(|c| c.name == "scan"));

        let scan = export_command_reference(&["scan".into()]).expect("scan");
        assert_eq!(scan.command.name, "scan");
        assert!(scan
            .command
            .arguments
            .iter()
            .any(|a| a.id == "i_own_this" || a.long.as_deref() == Some("--i-own-this")));
    }
}
