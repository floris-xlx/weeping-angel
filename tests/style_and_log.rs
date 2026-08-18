//! Style / log-http mode helpers (no TTY required).

use weeping_angel::finding::Severity;
use weeping_angel::parse::LogHttp;
use weeping_angel::style;

#[test]
fn log_http_mode_roundtrip() {
    for mode in [
        LogHttp::Full,
        LogHttp::Compact,
        LogHttp::Summary,
        LogHttp::Off,
    ] {
        style::set_log_http(mode);
        assert_eq!(style::log_http_mode(), mode);
    }
}

#[test]
fn terminal_width_clamp() {
    assert_eq!(style::terminal_width(40), 60); // min clamp
    assert_eq!(style::terminal_width(100), 100);
    assert_eq!(style::terminal_width(9999), 240); // max clamp
    let auto = style::terminal_width(0);
    assert!((60..=240).contains(&auto));
}

#[test]
fn truncate_url() {
    let long = format!("https://example.com/{}", "a".repeat(200));
    let t = style::truncate_url(&long, 40);
    assert!(t.chars().count() <= 40);
    assert!(t.ends_with('…') || t.len() < long.len());
    assert_eq!(style::truncate_url("short", 40), "short");
}

#[test]
fn format_bytes_and_ms() {
    assert_eq!(style::format_bytes(500), "500B");
    assert!(style::format_bytes(2048).contains("KB"));
    assert!(style::format_bytes(3 * 1024 * 1024).contains("MB"));
    assert_eq!(style::format_ms(50), "50ms");
    assert!(style::format_ms(2500).contains("s"));
}

#[test]
fn severity_badge_and_name_do_not_panic() {
    for s in [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Info,
    ] {
        let _ = style::severity_badge(s);
        let _ = style::severity_name(s);
    }
    let heat = style::severity_heat(1, 2, 3, 4, 5);
    assert!(!heat.is_empty());
}

#[test]
fn http_method_and_status_colors() {
    for m in ["GET", "POST", "DELETE", "OPTIONS", "TRACE"] {
        let _ = style::http_method(m);
    }
    for c in [200u16, 301, 404, 500, 0] {
        let _ = style::http_status(c);
    }
}

#[test]
fn log_request_respects_off_mode() {
    style::set_log_http(LogHttp::Off);
    style::log_request_ok(1, "GET", "https://x/", 200, 10, 100, None);
    style::set_log_http(LogHttp::Compact);
    style::log_request_ok(
        2,
        "GET",
        "https://x/long/path/here",
        200,
        10,
        100,
        Some("https://x/y"),
    );
    style::log_request_err(3, "GET", "https://x/", 5, "boom");
    style::log_progress("phase: test");
}

#[test]
fn paint_helpers() {
    let _ = style::brand("weeping-angel");
    let _ = style::ok("ok");
    let _ = style::warn("warn");
    let _ = style::err("err");
    let _ = style::phase("phase");
    let _ = style::rule(80, '═');
    let _ = style::section_title(80, "findings");
}
