//! The tab and the manifest agree: the tab asks Hangar for nothing its
//! permissions do not allow, and for nothing it does not use.

mod common;

use std::collections::BTreeSet;
use std::path::Path;

use common::*;
use serde_json::{Value, json};

use hangar_kit_sdk::api::{self, Needs};

fn manifest() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../hangar-kit.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the manifest")).expect("JSON")
}

fn permissions() -> Vec<String> {
    manifest()["tabs"][0]["permissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .map(|permission| permission.as_str().expect("a name").to_owned())
        .collect()
}

#[test]
fn the_tests_grant_what_the_manifest_declares() {
    let declared: BTreeSet<_> = permissions().into_iter().collect();
    let granted: BTreeSet<_> = PERMISSIONS.iter().map(|p| (*p).to_owned()).collect();
    assert_eq!(declared, granted);
}

#[test]
fn the_tab_keeps_its_page_and_names_its_module() {
    let manifest = manifest();
    let tab = &manifest["tabs"][0];
    assert_eq!(manifest["schema"], 1);
    assert_eq!(tab["entry"], "tabs/deps/index.html");
    assert_eq!(tab["native"], "tabs/deps.wasm");
    let module = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(tab["native"].as_str().unwrap());
    let bytes = std::fs::read(module).expect("the module is committed");
    assert_eq!(bytes[..4], *b"\0asm");
}

#[test]
fn every_call_the_tab_makes_is_one_its_permissions_allow_and_each_permission_is_used() {
    let granted = permissions();
    let mut session = open(
        Fake::default()
            .files(LAST, mixed())
            .panes(vec![claude("Jack", json!({ "role": "orchestrator" }))])
            .audit(printed(&mixed(), json!({}))),
    );
    session.press("Run again");
    session.hand_over(0);
    session.tab.event("panes.changed", json!({}));
    session
        .tab
        .event("theme", json!({ "scheme": "dark", "locale": "fr" }));
    session.settle();

    let mut used = BTreeSet::new();
    for (method, params) in &session.fake.calls {
        assert!(
            api::may_call(method, params, &granted),
            "{method} {params} is not allowed"
        );
        match api::needs(method) {
            Some(Needs::Permission(permission)) => {
                used.insert(permission.to_owned());
            }
            Some(Needs::Command) => {
                used.insert(format!("command:{}", params["name"].as_str().unwrap()));
            }
            _ => {}
        }
    }
    assert_eq!(used, granted.into_iter().collect::<BTreeSet<_>>());
}
