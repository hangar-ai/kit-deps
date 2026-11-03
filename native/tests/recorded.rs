//! The tab against what the audit script really prints: documents made by
//! tools/audit.js from the recorded outputs of npm, pnpm, cargo and pip
//! (test/native-fixtures.test.js keeps them what the script prints), not by
//! hand. The script and the tab are two halves of one document.

mod common;

use common::*;
use serde_json::{Value, json};

const NPM: &str = include_str!("fixtures/audit-npm.json");
const MULTI: &str = include_str!("fixtures/audit-multi.json");

fn value(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

#[test]
fn what_the_script_prints_is_read_whole_and_kept_as_it_printed_it() {
    for text in [NPM, MULTI] {
        let printed = value(text);
        let result = parse_result(text).expect("an audit");
        assert_eq!(result.at, AT);
        // What the tab keeps is the document it read, field for field: the
        // agents' skill reads it, and the tab reads it back when it opens.
        assert_eq!(result.document(), printed);
        assert_eq!(parse_result(&result.document().to_string()), Some(result));
    }
}

#[test]
fn an_npm_audit_is_listed_most_severe_first_with_its_versions() {
    let session = showing(value(NPM));
    assert_eq!(session.rows().len(), 9 + 3);
    let rows = session.rows();
    assert_eq!(rows[0].0, "minimist 1.2.0 → 1.2.6");
    assert_eq!(
        rows[0].1.as_deref(),
        Some("Critical · Prototype Pollution in minimist")
    );
    let text = session.text();
    assert!(text.contains("5 high or critical"));
    assert!(text.contains("Outdated · Minor"));
    assert!(
        !text.contains("Outdated · Major"),
        "no gap is listed that has no package"
    );
    assert_eq!(
        session.badge().unwrap(),
        json!({ "state": "failure", "text": "5 high" })
    );
    // One manager: nothing says which.
    assert!(!text.contains("· npm"));
}

#[test]
fn an_audit_of_three_managers_says_which_each_finding_is_of() {
    let session = showing(value(MULTI));
    let result = session.kit().result.as_ref().unwrap();
    assert_eq!(result.managers.len(), 3);
    assert_eq!(session.rows().len(), (9 + 1 + 1) + (4 + 3 + 2));
    let text = session.text();
    for manager in ["pnpm", "cargo", "pip"] {
        assert!(text.contains(&format!("· {manager}")), "{manager}");
    }
    assert!(
        text.contains("Not graded"),
        "RustSec and pip-audit do not grade"
    );
    assert!(text.contains("Outdated · Major"));
    assert_eq!(session.badge().unwrap()["state"], "failure");
}

#[test]
fn running_on_the_scripts_own_output_keeps_what_it_printed() {
    for text in [NPM, MULTI] {
        let mut session = open(Fake::default().audit(printed(&value(text), json!({}))));
        session.press("Run the audit");
        assert!(session.kit().error.is_none());
        assert_eq!(value(&session.fake.files[LAST]), value(text));
        // And the next opening reads it as it found it.
        let again = open(Fake::default().files(LAST, &session.fake.files[LAST]));
        assert_eq!(again.kit().result, session.kit().result);
    }
}

#[test]
fn the_brief_for_a_recorded_advisory_names_what_the_script_found() {
    let mut session = showing(value(NPM));
    session.hand_over(0);
    let sent = session.fake.calls_to("panes.send");
    let brief = sent[0]["text"].as_str().unwrap();
    assert!(brief.contains("move minimist to a version that is not affected (1.2.6 or later)"));
    assert!(brief.contains("Advisory: GHSA-xvch-5gv4-984h Prototype Pollution in minimist"));
    assert!(brief.contains("Affected versions: >=1.0.0 <1.2.6"));
    assert!(brief.contains("Link: https://github.com/advisories/GHSA-xvch-5gv4-984h"));
}
