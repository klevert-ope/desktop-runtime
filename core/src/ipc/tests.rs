//! Unit tests for IPC parsing, commands, and semver.

use super::*;
use updates::semver_compare;

#[test]
fn parse_message_valid_ping() {
    let raw = r#"{"id":"abc-123","name":"Ping"}"#;
    let env = parse_message(raw).expect("valid");
    assert_eq!(env.id, "abc-123");
    assert!(matches!(env.command, Command::Ping));
}

#[test]
fn parse_message_invalid_returns_none() {
    assert!(parse_message("").is_none());
    assert!(parse_message("{}").is_none());
    assert!(parse_message("not json").is_none());
}

#[test]
fn semver_compare_equal() {
    assert_eq!(semver_compare("1.0.0", "1.0.0"), 0);
    assert_eq!(semver_compare("0.0.0", "0.0.0"), 0);
}

#[test]
fn semver_compare_greater() {
    assert_eq!(semver_compare("2.0.0", "1.0.0"), 1);
    assert_eq!(semver_compare("1.1.0", "1.0.0"), 1);
    assert_eq!(semver_compare("1.0.1", "1.0.0"), 1);
}

#[test]
fn semver_compare_less() {
    assert_eq!(semver_compare("1.0.0", "2.0.0"), -1);
    assert_eq!(semver_compare("1.0.0", "1.1.0"), -1);
    assert_eq!(semver_compare("1.0.0", "1.0.1"), -1);
}

#[test]
fn is_blocking_command_identifies_blocking_commands() {
    assert!(super::is_blocking_command(&Command::OpenFileDialog));
    assert!(super::is_blocking_command(&Command::CheckForUpdates));
    assert!(super::is_blocking_command(&Command::OpenUrl {
        url: "https://example.com".to_string()
    }));
    assert!(!super::is_blocking_command(&Command::Ping));
    assert!(!super::is_blocking_command(&Command::ReadConfig));
    assert!(!super::is_blocking_command(&Command::GetVersion));
    assert!(!super::is_blocking_command(&Command::GetSystemInfo));
}

#[test]
fn open_url_rejects_non_http() {
    let cmd = Command::OpenUrl {
        url: "file:///etc/passwd".to_string(),
    };
    assert!(handle_command(&cmd).is_err());
    let cmd = Command::OpenUrl {
        url: "javascript:alert(1)".to_string(),
    };
    assert!(handle_command(&cmd).is_err());
}
