//! ANSI terminal styling for live scan output and reports.
//!
//! Respects `NO_COLOR` and only emits SGR codes when stderr looks like a TTY
//! (unless `FORCE_COLOR` / `CLICOLOR_FORCE` is set).

use std::io::{self, IsTerminal, Write};
use std::sync::OnceLock;

use crate::finding::Severity;

static COLOR: OnceLock<bool> = OnceLock::new();

/// Enable virtual terminal processing on Windows and cache color preference.
pub fn init() {
    let _ = color_enabled();
    #[cfg(windows)]
    enable_windows_ansi();
}

#[cfg(windows)]
fn enable_windows_ansi() {
    // Best-effort: enable VT processing so SGR colors work in Windows consoles.
    type Handle = *mut std::ffi::c_void;
    type Dword = u32;
    const STD_ERROR_HANDLE: Dword = 0xFFFF_FFF4; // (DWORD)-12
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: Dword = 0x0004;
    const ENABLE_PROCESSED_OUTPUT: Dword = 0x0001;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(n: Dword) -> Handle;
        fn GetConsoleMode(h: Handle, mode: *mut Dword) -> i32;
        fn SetConsoleMode(h: Handle, mode: Dword) -> i32;
    }

    // SAFETY: standard console-mode toggle for the process stderr handle.
    unsafe {
        let h = GetStdHandle(STD_ERROR_HANDLE);
        if h.is_null() || h == (-1isize as Handle) {
            return;
        }
        let mut mode: Dword = 0;
        if GetConsoleMode(h, &mut mode) == 0 {
            return;
        }
        let _ = SetConsoleMode(
            h,
            mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING | ENABLE_PROCESSED_OUTPUT,
        );
    }
}

pub fn color_enabled() -> bool {
    *COLOR.get_or_init(|| {
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        if std::env::var_os("FORCE_COLOR").is_some()
            || std::env::var_os("CLICOLOR_FORCE").is_some()
        {
            return true;
        }
        io::stderr().is_terminal()
    })
}

fn paint(code: &str, text: &str) -> String {
    if color_enabled() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn bold(text: &str) -> String {
    paint("1", text)
}
pub fn dim(text: &str) -> String {
    paint("2", text)
}
pub fn red(text: &str) -> String {
    paint("31", text)
}
pub fn green(text: &str) -> String {
    paint("32", text)
}
pub fn yellow(text: &str) -> String {
    paint("33", text)
}
pub fn blue(text: &str) -> String {
    paint("34", text)
}
pub fn magenta(text: &str) -> String {
    paint("35", text)
}
pub fn cyan(text: &str) -> String {
    paint("36", text)
}
pub fn bright_red(text: &str) -> String {
    paint("91", text)
}
pub fn bright_green(text: &str) -> String {
    paint("92", text)
}
pub fn bright_yellow(text: &str) -> String {
    paint("93", text)
}
pub fn bright_blue(text: &str) -> String {
    paint("94", text)
}
pub fn bright_magenta(text: &str) -> String {
    paint("95", text)
}
pub fn bright_cyan(text: &str) -> String {
    paint("96", text)
}
pub fn white(text: &str) -> String {
    paint("97", text)
}
pub fn bg_red(text: &str) -> String {
    paint("41;1;97", text)
}
pub fn bg_yellow(text: &str) -> String {
    paint("43;1;30", text)
}

/// Flush a line to stderr.
pub fn eprint_line(line: &str) {
    let _ = writeln!(io::stderr(), "{line}");
    let _ = io::stderr().flush();
}

pub fn brand(text: &str) -> String {
    bold(&magenta(text))
}

pub fn phase(text: &str) -> String {
    bold(&bright_cyan(text))
}

pub fn ok(text: &str) -> String {
    bright_green(text)
}

pub fn warn(text: &str) -> String {
    bright_yellow(text)
}

pub fn err(text: &str) -> String {
    bright_red(text)
}

/// Colored severity badge, e.g. `[CRIT]`.
pub fn severity_badge(s: Severity) -> String {
    let label = match s {
        Severity::Critical => " CRIT ",
        Severity::High => " HIGH ",
        Severity::Medium => " MED  ",
        Severity::Low => " LOW  ",
        Severity::Info => " INFO ",
    };
    if !color_enabled() {
        return format!("[{}]", label.trim());
    }
    match s {
        Severity::Critical => bg_red(label),
        Severity::High => paint("1;91", &format!("[{}]", label.trim())),
        Severity::Medium => paint("1;93", &format!("[{}]", label.trim())),
        Severity::Low => paint("1;94", &format!("[{}]", label.trim())),
        Severity::Info => paint("2;96", &format!("[{}]", label.trim())),
    }
}

pub fn severity_name(s: Severity) -> String {
    let name = s.as_str();
    if !color_enabled() {
        return name.to_string();
    }
    match s {
        Severity::Critical => bg_red(&format!(" {name} ")),
        Severity::High => bright_red(name),
        Severity::Medium => bright_yellow(name),
        Severity::Low => bright_blue(name),
        Severity::Info => dim(name),
    }
}

pub fn http_method(method: &str) -> String {
    match method {
        "GET" | "HEAD" => bright_green(method),
        "POST" | "PUT" | "PATCH" => bright_yellow(method),
        "DELETE" => bright_red(method),
        "OPTIONS" => cyan(method),
        _ => white(method),
    }
}

pub fn http_status(code: u16) -> String {
    let s = code.to_string();
    match code {
        200..=299 => bright_green(&s),
        300..=399 => bright_cyan(&s),
        400..=499 => bright_yellow(&s),
        500..=599 => bright_red(&s),
        _ => dim(&s),
    }
}

pub fn format_bytes(n: usize) -> String {
    if n < 1024 {
        format!("{n}B")
    } else if n < 1024 * 1024 {
        format!("{:.1}KB", n as f64 / 1024.0)
    } else {
        format!("{:.1}MB", n as f64 / (1024.0 * 1024.0))
    }
}

pub fn format_ms(ms: u128) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", ms as f64 / 1000.0)
    }
}

/// Live request line after a successful HTTP exchange.
pub fn log_request_ok(
    n: u64,
    method: &str,
    url: &str,
    status: u16,
    elapsed_ms: u128,
    body_len: usize,
    final_url: Option<&str>,
) {
    let arrow = dim("→");
    let back = dim("←");
    let n_s = dim(&format!("#{n}"));
    let meth = http_method(method);
    let st = http_status(status);
    let time = dim(&format_ms(elapsed_ms));
    let size = dim(&format_bytes(body_len));
    let mut line = format!("{arrow} {n_s} {meth} {url}");
    eprint_line(&line);
    line = format!("  {back} {st}  {time}  {size}");
    if let Some(fu) = final_url {
        if fu != url {
            line.push_str(&format!("  {} {}", dim("redir→"), dim(fu)));
        }
    }
    eprint_line(&line);
}

/// Live request line after a failed HTTP exchange.
pub fn log_request_err(n: u64, method: &str, url: &str, elapsed_ms: u128, error: &str) {
    let arrow = dim("→");
    let bad = err("✗");
    let n_s = dim(&format!("#{n}"));
    let meth = http_method(method);
    let time = dim(&format_ms(elapsed_ms));
    eprint_line(&format!("{arrow} {n_s} {meth} {url}"));
    eprint_line(&format!("  {bad} {}  {time}  {}", err("ERR"), err(error)));
}

/// Phase / progress banner used by the engine.
pub fn log_progress(msg: &str) {
    let tag = brand("[weeping-angel]");
    eprint_line(&format!("{tag} {}", phase(msg)));
}
