//! No secret is committed (E14). The tracked files (`git ls-files`) are scanned for the shapes of
//! well-known credentials: cloud and API keys, access tokens, private key blocks. It runs with the
//! engine's tests, so every CI run checks it without a CI job of its own (CI stays shared).
//!
//! A match fails with the file and the kind, never the value. A test fixture that must hold such a
//! shape (none today) goes in `ALLOWED` with the reason.

use regex::Regex;

/// Files that may hold a matching shape on purpose.
const ALLOWED: &[&str] = &[];
/// Larger files are data (fonts, images), not where a key is pasted.
const MAX_SCANNED: u64 = 2 * 1024 * 1024;

/// (kind, pattern). Built from parts so this file doesn't match itself.
fn shapes() -> Vec<(&'static str, Regex)> {
    let p = |s: &[&str]| s.concat();
    [
        ("Anthropic API key", p(&["sk-", "ant-", r"[A-Za-z0-9_\-]{20,}"])),
        ("OpenAI-style API key", p(&["sk-", r"(proj-)?[A-Za-z0-9]{32,}"])),
        ("GitHub token", p(&["gh", r"[pousr]_[A-Za-z0-9]{36,}"])),
        ("GitHub fine-grained token", p(&["github", "_pat_", r"[A-Za-z0-9_]{40,}"])),
        ("AWS access key", p(&["AK", r"IA[0-9A-Z]{16}"])),
        ("Slack token", p(&["xo", r"x[abprs]-[A-Za-z0-9\-]{10,}"])),
        ("Google API key", p(&["AI", r"za[0-9A-Za-z_\-]{35}"])),
        ("Figma token", p(&["fig", r"d_[A-Za-z0-9_\-]{30,}"])),
        ("Notion token", p(&["ntn", r"_[A-Za-z0-9]{40,}"])),
        ("Atlassian API token", p(&["ATAT", r"T3xFfGF0[A-Za-z0-9_\-=]{20,}"])),
        ("private key block", p(&["-----BEGIN ", r"(RSA |EC |DSA |OPENSSH |ENCRYPTED )?PRIVATE KEY-----"])),
    ]
    .into_iter()
    .filter_map(|(k, re)| Regex::new(&re).ok().map(|r| (k, r)))
    .collect()
}

#[test]
fn no_credentials_are_committed() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(out) = std::process::Command::new("git").arg("-C").arg(&root).args(["ls-files", "-z"]).output() else {
        eprintln!("git is not available here; skipping");
        return;
    };
    if !out.status.success() {
        eprintln!("not a git checkout; skipping");
        return;
    }
    let shapes = shapes();
    assert_eq!(shapes.len(), 11, "every pattern compiles");
    let mut found = Vec::new();
    for file in out.stdout.split(|b| *b == 0).filter(|f| !f.is_empty()) {
        let name = String::from_utf8_lossy(file).into_owned();
        if ALLOWED.contains(&name.as_str()) {
            continue;
        }
        let path = root.join(&name);
        if std::fs::metadata(&path).map_or(true, |m| !m.is_file() || m.len() > MAX_SCANNED) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        for (kind, re) in &shapes {
            if re.is_match(&text) {
                found.push(format!("{name}: {kind}"));
            }
        }
    }
    assert!(found.is_empty(), "credential-shaped text committed (remove it and rotate the key):\n{}", found.join("\n"));
}

#[test]
fn the_shapes_catch_what_they_are_for() {
    let shapes = shapes();
    let caught = |s: &str| shapes.iter().any(|(_, re)| re.is_match(s));
    let p = |s: &[&str]| s.concat();
    assert!(caught(&p(&["AK", "IAIOSFODNN7EXAMPLE"])));
    assert!(caught(&p(&["gh", "p_", &"a".repeat(36)])));
    assert!(caught(&p(&["-----BEGIN ", "OPENSSH PRIVATE KEY-----"])));
    assert!(caught(&p(&["sk-", "ant-", "api03-", &"b".repeat(30)])));
    // Ordinary text and token names of the design system are not secrets.
    assert!(!caught("Colors/Surface/Background sk-tokens AKIA"));
    assert!(!caught("--colors-surface-brand-background: var(--x);"));
}
