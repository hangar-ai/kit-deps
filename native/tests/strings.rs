//! The words: English and French hold the same keys, the same placeholders,
//! and French has its accents (test/manifest.test.js).

use std::collections::BTreeSet;

use kit_deps_tab::{EN, FR, strings};

/// The keys without the plural suffix: `stateHigh_one` and `_other` are one.
fn keys(table: &[(&str, &str)]) -> BTreeSet<String> {
    table
        .iter()
        .map(|(key, _)| {
            key.trim_end_matches("_one")
                .trim_end_matches("_other")
                .to_owned()
        })
        .collect()
}

fn placeholders(template: &str) -> Vec<String> {
    let mut found: Vec<String> = template
        .split('{')
        .skip(1)
        .filter_map(|rest| rest.split_once('}').map(|(name, _)| format!("{{{name}}}")))
        .collect();
    found.sort();
    found
}

#[test]
fn english_and_french_hold_the_same_keys_and_the_same_placeholders() {
    assert_eq!(keys(EN), keys(FR));
    let find = |table: &'static [(&'static str, &'static str)], key: &str| {
        table
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, text)| *text)
    };
    for (key, english) in EN {
        let french = find(FR, key).unwrap_or_else(|| panic!("{key} has no French"));
        assert_eq!(placeholders(french), placeholders(english), "{key}");
    }
    for table in [EN, FR] {
        let names: BTreeSet<_> = table.iter().map(|(key, _)| key).collect();
        assert_eq!(names.len(), table.len(), "a key twice");
    }
}

#[test]
fn french_has_its_accents() {
    let french = strings("fr");
    assert_eq!(french.t("tab"), "Dépendances");
    assert_eq!(french.t("handOver"), "Confier à un agent");
    assert!(french.t("deniedText").contains("Réglages › Kits"));
    let all: String = FR
        .iter()
        .map(|(_, text)| *text)
        .collect::<Vec<_>>()
        .join(" ");
    for wrong in ["Reglages", "echec", "Dependances", "a un agent"] {
        assert!(!all.contains(wrong), "{wrong}");
    }
}

#[test]
fn counts_pick_their_form_in_both_languages() {
    let english = strings("en");
    assert_eq!(english.tn("stateVulns", 1, &[]), "1 vulnerability");
    assert_eq!(english.tn("stateVulns", 3, &[]), "3 vulnerabilities");
    assert_eq!(english.tn("badgeHigh", 1, &[]), "1 high");
    let french = strings("fr");
    assert_eq!(french.tn("stateVulns", 1, &[]), "1 faille");
    assert_eq!(french.tn("stateVulns", 3, &[]), "3 failles");
    assert_eq!(french.tn("badgeHigh", 1, &[]), "1 grave");
    assert_eq!(french.tn("badgeHigh", 4, &[]), "4 graves");
}

#[test]
fn a_placeholder_nobody_filled_is_left_as_it_is() {
    assert_eq!(strings("en").tv("sentTo", &[]), "Handed to {agent}");
}
