//! Independent command excerpts, not a prompt-injection recall corpus.
//! See docs/PI079-EVIDENCE.md for provenance and sampling limitations.
use injection_scanner::allowlist::Suppressions;
use injection_scanner::pattern::Severity;
use injection_scanner::patterns::load_embedded_patterns;
use injection_scanner::scanner::Scanner;
use serde::Deserialize;

mod common;

#[derive(Deserialize)]
struct Specimen {
    name: String,
    text: String,
    expected_match: bool,
}

fn specimens() -> Vec<Specimen> {
    let cases: Vec<Specimen> =
        serde_json::from_str(include_str!("fixtures/pi079-evidence.json")).unwrap();
    assert_eq!(
        cases.len(),
        3,
        "update the measured table when specimens change"
    );
    cases
}

fn fires(scanner: &Scanner, text: &str) -> bool {
    let report = scanner.scan("specimen.txt", text, &Suppressions::default());
    report
        .matches
        .iter()
        .chain(&report.low_confidence)
        .any(|m| m.pattern_id == "PI079")
}

#[test]
fn independent_commands_record_the_benign_overlap() {
    for specimen in specimens() {
        assert_eq!(
            fires(common::scanner(), &specimen.text),
            specimen.expected_match,
            "{}",
            specimen.name
        );
    }
    let categories = load_embedded_patterns().unwrap();
    let pattern = categories
        .iter()
        .flat_map(|c| &c.patterns)
        .find(|p| p.id == "PI079")
        .unwrap();
    assert_eq!(pattern.severity, Some(Severity::Medium));
}

#[test]
fn threshold_sensitivity_on_independent_commands() {
    // Counts refer to base64 characters AFTER the literal AAAA prefix.
    // Raising the cutoff above 64 suppresses the legitimate Ed25519 example,
    // but also excludes real keys of that size regardless of provenance.
    let cases = specimens();
    for (minimum, expected) in [
        (8, [true, true, true]),
        (40, [true, true, false]),
        (64, [true, true, false]),
        (65, [true, false, false]),
        (128, [true, false, false]),
        (512, [false, false, false]),
    ] {
        let mut categories = load_embedded_patterns().unwrap();
        for category in &mut categories {
            category.patterns.retain(|p| p.id == "PI079");
            for pattern in &mut category.patterns {
                assert_eq!(pattern.pattern.matches("{40,}").count(), 1);
                pattern.pattern = pattern.pattern.replace("{40,}", &format!("{{{minimum},}}"));
            }
        }
        let scanner = Scanner::new(&categories).unwrap();
        for (specimen, expected) in cases.iter().zip(expected) {
            assert_eq!(
                fires(&scanner, &specimen.text),
                expected,
                "minimum={minimum}: {}",
                specimen.name
            );
        }
    }
}

#[test]
fn threshold_boundary_is_44_total_base64_characters() {
    // Synthetic probes pin mechanics only; they are not independent evidence.
    for (after_prefix, expected) in [(39, false), (40, true), (41, true)] {
        let text = format!(
            "echo 'ssh-ed25519 AAAA{}' >> ~/.ssh/authorized_keys",
            "B".repeat(after_prefix)
        );
        assert_eq!(fires(common::scanner(), &text), expected, "{after_prefix}");
    }
}
