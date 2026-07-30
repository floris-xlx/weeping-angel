//! Docs export binary surface via library API.

use weeping_angel::docs_export::export_command_reference;

#[test]
fn root_export_has_scan_subcommand() {
    let exp = export_command_reference(&[]).unwrap();
    assert_eq!(exp.generated_by, "weeping-angel-docs-export");
    assert!(!exp.version.is_empty());
    assert_eq!(exp.command.name, "weeping-angel");
    assert!(
        exp.command.subcommands.iter().any(|c| c.name == "scan"),
        "subcommands={:?}",
        exp.command
            .subcommands
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn scan_export_includes_consent_and_targets() {
    let exp = export_command_reference(&["scan".into()]).unwrap();
    assert_eq!(exp.command.name, "scan");
    assert!(exp.command.display_name.contains("scan"));
    let ids: Vec<_> = exp.command.arguments.iter().map(|a| a.id.as_str()).collect();
    assert!(
        ids.iter().any(|id| id.contains("i_own") || id.contains("own")),
        "ids={ids:?}"
    );
    assert!(
        exp.command
            .arguments
            .iter()
            .any(|a| a.long.as_deref() == Some("--i-own-this")
                || a.display.contains("i-own-this")),
        "args={:?}",
        exp.command.arguments.iter().map(|a| &a.display).collect::<Vec<_>>()
    );
    assert!(
        exp.command
            .arguments
            .iter()
            .any(|a| a.long.as_deref() == Some("--allow-host")
                || a.display.contains("allow-host")),
    );
    assert!(
        exp.command.arguments.iter().any(|a| a.kind == "positional"),
        "expected targets positional"
    );
}

#[test]
fn scan_export_mentions_fast_and_log_http() {
    let exp = export_command_reference(&["scan".into()]).unwrap();
    let displays: String = exp
        .command
        .arguments
        .iter()
        .map(|a| a.display.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(displays.contains("fast") || displays.contains("--fast"), "{displays}");
    assert!(
        displays.contains("log-http") || displays.contains("log_http"),
        "{displays}"
    );
}

#[test]
fn unknown_path_errors() {
    let err = export_command_reference(&["does-not-exist".into()]).unwrap_err();
    assert!(err.contains("Unknown") || err.contains("does-not-exist"), "{err}");
}

#[test]
fn json_serialize_export() {
    let exp = export_command_reference(&["scan".into()]).unwrap();
    let s = serde_json::to_string_pretty(&exp).unwrap();
    assert!(s.contains("weeping-angel"));
    assert!(s.contains("arguments"));
}
