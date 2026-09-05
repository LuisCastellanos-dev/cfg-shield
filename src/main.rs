//! cfg-shield — Feature Flag Semantic Drift Detector
//!
//! Detects the Rust equivalent of COBOL col7='*' dormant code:
//! features that gate security-sensitive code but are not in `default`,
//! causing CI to pass with 0 tests while production builds activate
//! untested code paths.
//!
//! Classification follows VTR Audit Master Prompt v3.5:
//!   CONFIRMADO  — 0 tests default, N tests with feature
//!   PROBABLE    — feature gates security-sensitive code, not tested in default
//!   PROYECCION  — feature present, impact not yet demonstrated
//!
//! Usage: cfg-shield <path-to-crate>

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Finding {
    rule: &'static str,
    feature: String,
    classification: Classification,
    observation: String,
    evidence: String,
}

#[derive(Debug, PartialEq)]
enum Classification {
    Confirmado,
    Probable,
    Proyeccion,
}

impl std::fmt::Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Confirmado  => write!(f, "CONFIRMADO"),
            Self::Probable    => write!(f, "PROBABLE"),
            Self::Proyeccion  => write!(f, "PROYECCION"),
        }
    }
}

#[derive(Debug)]
struct CrateInfo {
    path: PathBuf,
    name: String,
    default_features: Vec<String>,
    all_features: Vec<String>,
    non_default_features: Vec<String>,
}

// ── Keywords for security-sensitive code ─────────────────────────────────────

const SECURITY_KEYWORDS: &[&str] = &[
    "verify", "validate", "sign", "encrypt", "decrypt",
    "auth", "secret", "private", "proof", "cipher",
    "hash", "hmac", "signature", "certificate", "tls",
    "crypto", "zero", "password", "token", "key",
];

// ── Cargo.toml parsing ────────────────────────────────────────────────────────

fn parse_crate(crate_path: &Path) -> Result<CrateInfo, String> {
    let cargo_toml = crate_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(format!("No Cargo.toml found at {}", crate_path.display()));
    }

    let content = std::fs::read_to_string(&cargo_toml)
        .map_err(|e| format!("Cannot read Cargo.toml: {e}"))?;

    let parsed: toml::Value = content.parse()
        .map_err(|e| format!("Cannot parse Cargo.toml: {e}"))?;

    let name = parsed.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown")
        .to_string();

    let features_table = match parsed.get("features").and_then(|f| f.as_table()) {
        Some(t) => t,
        None => return Ok(CrateInfo {
            path: crate_path.to_path_buf(),
            name,
            default_features: vec![],
            all_features: vec![],
            non_default_features: vec![],
        }),
    };

    let default_features: Vec<String> = features_table
        .get("default")
        .and_then(|d| d.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect())
        .unwrap_or_default();

    let all_features: Vec<String> = features_table
        .keys()
        .filter(|k| *k != "default")
        .cloned()
        .collect();

    let non_default_features: Vec<String> = all_features
        .iter()
        .filter(|f| !default_features.contains(f))
        .cloned()
        .collect();

    Ok(CrateInfo {
        path: crate_path.to_path_buf(),
        name,
        default_features,
        all_features,
        non_default_features,
    })
}

// ── Source analysis ───────────────────────────────────────────────────────────

/// Find features that gate security-sensitive code
fn find_security_gated_features(crate_info: &CrateInfo) -> HashMap<String, Vec<String>> {
    let mut feature_contexts: HashMap<String, Vec<String>> = HashMap::new();

    let src_dir = crate_info.path.join("src");
    if !src_dir.exists() {
        return feature_contexts;
    }

    let rs_files = find_rs_files(&src_dir);

    for fpath in rs_files {
        let Ok(content) = std::fs::read_to_string(&fpath) else { continue };
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if !line.contains("cfg(feature") && !line.contains("cfg(not(feature") {
                continue;
            }

            // Extract feature name
            if let Some(feat) = extract_feature_name(line) {
                if !crate_info.non_default_features.contains(&feat) {
                    continue;
                }

                // Check context for security keywords
                let start = i.saturating_sub(2);
                let end = (i + 8).min(lines.len());
                let context = lines[start..end].join("\n");

                if SECURITY_KEYWORDS.iter().any(|kw| {
                    context.to_lowercase().contains(kw)
                }) {
                    let fname = fpath.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    feature_contexts
                        .entry(feat)
                        .or_default()
                        .push(format!("{}:{}", fname, i + 1));
                }
            }
        }
    }

    feature_contexts
}

fn extract_feature_name(line: &str) -> Option<String> {
    let re_start = line.find("feature")?;
    let after = &line[re_start..];
    let quote_start = after.find('"')? + 1;
    let after_quote = &after[quote_start..];
    let quote_end = after_quote.find('"')?;
    Some(after_quote[..quote_end].to_string())
}

fn find_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_rs_files(&path));
            } else if path.extension().map_or(false, |e| e == "rs") {
                files.push(path);
            }
        }
    }
    files
}

// ── Test count differential ───────────────────────────────────────────────────

fn count_tests(crate_path: &Path, features: Option<&[String]>) -> Result<usize, String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test")
       .arg("--locked")
       .current_dir(crate_path);

    match features {
        None => { cmd.arg("--no-default-features"); }
        Some(feats) if feats.is_empty() => { cmd.arg("--no-default-features"); }
        Some(feats) => {
            cmd.arg("--no-default-features");
            cmd.arg("--features");
            cmd.arg(feats.join(","));
        }
    }

    // Capture test count from output
    let output = cmd.output()
        .map_err(|e| format!("cargo test failed: {e}"))?;

    if !output.status.success() {

        let stderr = String::from_utf8_lossy(&output.stderr);
        // Distinguish compilation failure from 0 tests executed.
        // count_tests previously returned Ok(0) for both cases —
        // indistinguishable and causes false CONFIRMADO classification.
        // VS-009: detected in audit session 2026-09-04.
        return Err(format!("cargo test failed (exit {:?}): {}",
            output.status.code(), stderr.lines().take(3).collect::<Vec<_>>().join(" | ")));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    // Parse "test result: ok. N passed"
    let mut total = 0usize;
    for line in combined.lines() {
        if line.contains("test result") && line.contains("passed") {
            if let Some(n) = parse_passed_count(line) {
                total += n;
            }
        }
    }

    Ok(total)
}

fn parse_passed_count(line: &str) -> Option<usize> {
    // "test result: ok. 5 passed; ..."
    let after_ok = line.split("ok.").nth(1)?;
    let trimmed = after_ok.trim();
    let num_str = trimmed.split_whitespace().next()?;
    num_str.parse().ok()
}

// ── Main scan ─────────────────────────────────────────────────────────────────

fn scan(crate_path: &Path) -> Vec<Finding> {
    let mut findings = vec![];

    println!("  Parsing Cargo.toml...");
    let crate_info = match parse_crate(crate_path) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("  ERROR: {e}");
            return findings;
        }
    };

    println!("  Crate: {}", crate_info.name);
    println!("  Default features: {:?}", crate_info.default_features);
    println!("  Non-default features: {:?}", crate_info.non_default_features);

    if crate_info.non_default_features.is_empty() {
        println!("  No non-default features — nothing to analyze.");
        return findings;
    }

    // Step 1: Find features gating security-sensitive code
    println!("\n  Scanning source for security-sensitive cfg(feature) gates...");
    let security_features = find_security_gated_features(&crate_info);

    if security_features.is_empty() {
        println!("  No security-sensitive feature gates found.");
    } else {
        for (feat, locations) in &security_features {
            println!("  [FOUND] feature='{}' gates security code at: {:?}", feat, locations);
        }
    }

    // Step 2: Test count differential
    println!("\n  Running test count differential...");
    println!("  [1/2] cargo test --no-default-features");
    let default_count = match count_tests(crate_path, None) {
        Ok(n) => n,
        Err(e) => {
            println!("  [WARN] Default build failed — compilation error, not 0 tests: {}", e);
            return findings;
        }
    };
    println!("        → {} tests executed", default_count);

    for feature in &crate_info.non_default_features {
        println!("  [2/N] cargo test --features {}", feature);
        let feat_count = match count_tests(crate_path, Some(&[feature.clone()])) {
            Ok(n) => n,
            Err(e) => {
                println!("  [WARN] Feature build failed for '{}': {}", feature, e);
                continue;
            }
        };
        println!("        → {} tests executed", feat_count);

        if feat_count > default_count {
            let delta = feat_count - default_count;
            let is_security = security_features.contains_key(feature);

            let classification = if default_count == 0 && is_security {
                Classification::Confirmado
            } else if is_security {
                Classification::Probable
            } else {
                Classification::Proyeccion
            };

            findings.push(Finding {
                rule: "CFG-R01",
                feature: feature.clone(),
                classification,
                observation: format!(
                    "Feature '{}' activates {} additional test(s) not run in default CI. \
                     Default: {} tests, with feature: {} tests. \
                     Security-sensitive code gated: {}. \
                     The feature flag lives outside the source file.",
                    feature, delta, default_count, feat_count, is_security
                ),
                evidence: format!(
                    "cargo test --no-default-features → {} passed | \
                     cargo test --features {} → {} passed | \
                     crate: {}",
                    default_count, feature, feat_count, crate_info.name
                ),
            });
        }
    }

    findings
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let crate_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        eprintln!("Usage: cfg-shield <path-to-crate>");
        eprintln!("Example: cfg-shield ~/generic-ecies");
        std::process::exit(1);
    };

    println!("═══════════════════════════════════════════════════════════");
    println!(" cfg-shield v0.1.0 — Feature Flag Semantic Drift Detector  ");
    println!(" Rust equivalent of COBOL col7 dormant code analysis       ");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Target: {}", crate_path.display());
    println!();

    let findings = scan(&crate_path);

    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!(" FINDINGS: {}", findings.len());
    println!("═══════════════════════════════════════════════════════════");

    if findings.is_empty() {
        println!(" No feature flag drift detected.");
        println!(" Detector limitation: only analyzes features with test count");
        println!(" differential. Build-time and runtime-only divergences");
        println!(" require Phase 2 analysis.");
    }

    for (i, f) in findings.iter().enumerate() {
        println!();
        println!(" Finding #{}", i + 1);
        println!(" Rule          : {}", f.rule);
        println!(" Feature       : {}", f.feature);
        println!(" Classification: {}", f.classification);
        println!(" Observation   : {}", f.observation);
        println!(" Evidence      : {}", f.evidence);
        println!(" Limitation    : Security keyword match is heuristic.");
        println!("                 Verify manually before escalating.");
    }

    println!();

    let confirmed = findings.iter()
        .filter(|f| f.classification == Classification::Confirmado)
        .count();

    if confirmed > 0 {
        println!(" {} CONFIRMADO finding(s) — test differential demonstrated.", confirmed);
        std::process::exit(1); // CI fail
    } else {
        std::process::exit(0);
    }
}
