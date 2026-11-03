//! Shared by the tests: a fake Hangar answering a tab's calls (test/helpers.js'
//! `fakeHangar`), the documents the audit script prints, and what a person
//! would read on the page.
#![allow(dead_code)]

use std::collections::BTreeMap;

use serde_json::{Value, json};

use hangar_kit_sdk::api::KitError;
use hangar_kit_sdk::testing::{self, Harness, StartExt};
use hangar_kit_sdk::ui::{ButtonKind, Element, HandlerId};

pub use kit_deps_tab::*;

/// `Date.parse("2026-10-31T17:02:00Z")`.
pub const AT: i64 = 1_793_466_120_000;
/// Five minutes after the audit.
pub const LATER: i64 = AT + 5 * 60 * 1000;

pub const PERMISSIONS: [&str; 5] = [
    "command:audit",
    "panes.read",
    "panes.send",
    "data.read",
    "data.write",
];

// ---- the documents ---------------------------------------------------------

/// `base` with the fields of `extra` set over it.
pub fn with(mut base: Value, extra: Value) -> Value {
    if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }
    base
}

pub fn vuln(extra: Value) -> Value {
    with(
        json!({
            "package": "lodash", "installed": "4.17.15", "severity": "high",
            "title": "Command Injection in lodash", "id": "GHSA-35jh-r3h4-6jhm",
            "url": "https://github.com/advisories/GHSA-35jh-r3h4-6jhm",
            "fixed": "4.17.21", "vulnerable": "<4.17.21"
        }),
        extra,
    )
}

pub fn old(extra: Value) -> Value {
    with(
        json!({ "package": "react", "current": "17.0.2", "latest": "19.1.0", "kind": "major", "dev": false }),
        extra,
    )
}

pub fn manager(extra: Value) -> Value {
    with(
        json!({
            "manager": "pnpm", "lockfile": "pnpm-lock.yaml", "status": "ok",
            "steps": { "audit": "ok", "outdated": "ok" }, "missing": [], "errors": [],
            "truncated": false, "vulnerabilities": [], "outdated": []
        }),
        extra,
    )
}

pub fn document(managers: Vec<Value>, extra: Value) -> Value {
    with(
        json!({ "schema": 1, "at": AT, "status": "ok", "counts": {}, "managers": managers }),
        extra,
    )
}

/// Two vulnerabilities that count (one critical, one high), one that does not,
/// and an outdated package of each gap, all out of order.
pub fn mixed() -> Value {
    document(
        vec![manager(json!({
            "vulnerabilities": [
                vuln(json!({ "severity": "low", "package": "a", "title": "Low one" })),
                vuln(json!({ "severity": "critical", "package": "minimist", "installed": "1.2.0", "fixed": "1.2.6", "title": "Prototype Pollution in minimist" })),
                vuln(json!({})),
            ],
            "outdated": [
                old(json!({ "kind": "patch", "package": "z", "current": "1.0.0", "latest": "1.0.1" })),
                old(json!({})),
                old(json!({ "kind": "minor", "package": "m", "current": "1.0.0", "latest": "1.1.0" })),
            ],
        }))],
        json!({}),
    )
}

pub fn claude(name: &str, extra: Value) -> Value {
    with(
        json!({ "name": name, "agent": "claude", "role": "worker", "state": "idle", "canSend": true }),
        extra,
    )
}

/// What `command.run` answers when audit.js printed `document`.
pub fn printed(document: &Value, extra: Value) -> Value {
    with(
        json!({ "exitCode": 0, "stdout": format!("{document}\n"), "stderr": "", "timedOut": false, "truncated": false }),
        extra,
    )
}

pub fn failure(code: &str) -> KitError {
    KitError::new(code, format!("could not: {code}"))
}

// ---- the fake Hangar -------------------------------------------------------

/// How `command.run` is answered.
pub enum Audit {
    /// Nothing was set up.
    Nothing,
    Answer(Value),
    Fails(KitError),
}

pub struct Fake {
    /// The kit's data folder.
    pub files: BTreeMap<String, String>,
    pub panes: Vec<Value>,
    pub audit: Audit,
    /// Methods answered `denied`.
    pub deny: Vec<&'static str>,
    /// What `panes.send` fails with.
    pub send_error: Option<&'static str>,
    /// Methods left waiting, to answer by hand.
    pub hold: Vec<&'static str>,
    /// Every call made, with its params, in order.
    pub calls: Vec<(String, Value)>,
}

impl Default for Fake {
    fn default() -> Self {
        Fake {
            files: BTreeMap::new(),
            panes: Vec::new(),
            audit: Audit::Nothing,
            deny: Vec::new(),
            send_error: None,
            hold: Vec::new(),
            calls: Vec::new(),
        }
    }
}

impl Fake {
    pub fn files(mut self, name: &str, text: impl ToString) -> Self {
        self.files.insert(name.to_owned(), text.to_string());
        self
    }

    pub fn panes(mut self, panes: Vec<Value>) -> Self {
        self.panes = panes;
        self
    }

    pub fn audit(mut self, answer: Value) -> Self {
        self.audit = Audit::Answer(answer);
        self
    }

    pub fn audit_fails(mut self, code: &str) -> Self {
        self.audit = Audit::Fails(failure(code));
        self
    }

    pub fn deny(mut self, method: &'static str) -> Self {
        self.deny.push(method);
        self
    }

    pub fn send_error(mut self, code: &'static str) -> Self {
        self.send_error = Some(code);
        self
    }

    pub fn hold(mut self, method: &'static str) -> Self {
        self.hold.push(method);
        self
    }

    /// What it answers to `method` with `params`.
    fn answer(&mut self, method: &str, params: &Value) -> Result<Value, KitError> {
        if self.deny.contains(&method) {
            return Err(KitError::new(
                KitError::DENIED,
                format!("this tab was not granted what {method} needs"),
            ));
        }
        let path = params["path"].as_str().unwrap_or_default().to_owned();
        match method {
            "tab.setTitle" | "tab.setBadge" | "ui.toast" => Ok(json!({})),
            "data.read" => match self.files.get(&path) {
                Some(text) => Ok(json!({ "text": text })),
                None => Err(failure(KitError::NOT_FOUND)),
            },
            "data.write" => {
                self.files
                    .insert(path, params["text"].as_str().unwrap_or_default().to_owned());
                Ok(json!({}))
            }
            "panes.list" => Ok(Value::Array(self.panes.clone())),
            "panes.send" => match self.send_error {
                Some(code) => Err(failure(code)),
                None => Ok(json!({ "queued": true, "pane": params["pane"] })),
            },
            "command.run" => match &self.audit {
                Audit::Answer(answer) => Ok(answer.clone()),
                Audit::Fails(error) => Err(error.clone()),
                Audit::Nothing => Err(failure(KitError::NOT_FOUND)),
            },
            other => Err(KitError::new(
                KitError::DENIED,
                format!("this tab was not granted what {other} needs"),
            )),
        }
    }

    pub fn calls_to(&self, method: &str) -> Vec<Value> {
        self.calls
            .iter()
            .filter(|(name, _)| name == method)
            .map(|(_, params)| params.clone())
            .collect()
    }
}

/// A tab started, and the Hangar answering it.
pub struct Session {
    pub tab: Harness<Deps>,
    pub fake: Fake,
}

/// A tab in English that holds every permission it asks for, its first calls
/// answered.
pub fn open(fake: Fake) -> Session {
    open_in(fake, "en")
}

pub fn open_in(fake: Fake, locale: &str) -> Session {
    testing::set_now(Some(LATER));
    let start = testing::start().grant(PERMISSIONS).locale(locale);
    let mut session = Session {
        tab: Harness::<Deps>::new(start),
        fake,
    };
    session.settle();
    session
}

/// A tab opened on the last audit `result`, a Claude Code agent to hand
/// findings to, five minutes after it ran.
pub fn showing(result: Value) -> Session {
    showing_in(result, "en")
}

pub fn showing_in(result: Value, locale: &str) -> Session {
    open_in(
        Fake::default()
            .files(LAST, result)
            .panes(vec![claude("Jack", json!({}))]),
        locale,
    )
}

impl Session {
    /// Answers the calls waiting, and those they bring, but the methods held.
    pub fn settle(&mut self) {
        loop {
            let waiting: Vec<_> = self
                .tab
                .calls()
                .iter()
                .filter(|call| !self.fake.hold.contains(&call.method.as_str()))
                .cloned()
                .collect();
            if waiting.is_empty() {
                return;
            }
            for call in waiting {
                let answer = self.fake.answer(&call.method, &call.params);
                self.fake
                    .calls
                    .push((call.method.clone(), call.params.clone()));
                self.tab.reply(call.id, answer);
            }
        }
    }

    /// Answers the call to `method` that was held.
    pub fn release(&mut self, method: &'static str) {
        self.fake.hold.retain(|held| *held != method);
        self.settle();
    }

    pub fn press(&mut self, label: &str) {
        self.tab.press(label);
        self.settle();
    }

    /// Everything a person reads on the page, one text a line, with the
    /// no-break spaces of the typography written as spaces.
    pub fn text(&self) -> String {
        self.tab
            .texts()
            .join("\n")
            .replace(['\u{a0}', '\u{202f}'], " ")
    }

    pub fn kit(&self) -> &Deps {
        self.tab.kit()
    }

    /// The buttons of the page, in order: label, kind, and whether they act.
    pub fn buttons(&self) -> Vec<Button> {
        let mut buttons = Vec::new();
        self.tab.view().root.for_each(&mut |element| {
            if let Element::Button(button) = element {
                buttons.push(Button {
                    label: button
                        .label
                        .clone()
                        .or_else(|| button.tooltip.clone())
                        .unwrap_or_default(),
                    kind: button.kind,
                    handler: button.on_press,
                });
            }
        });
        buttons
    }

    pub fn button(&self, label: &str) -> Button {
        self.buttons()
            .into_iter()
            .find(|button| button.label == label)
            .unwrap_or_else(|| panic!("no button {label:?}; there are {:?}", self.buttons()))
    }

    /// The buttons that hand a finding to an agent (or say they are sending).
    pub fn hand_overs(&self) -> Vec<Button> {
        self.buttons()
            .into_iter()
            .filter(|button| button.label == "Hand over to an agent" || button.label == "Sending…")
            .collect()
    }

    /// Presses the `index`th button that hands a finding over.
    pub fn hand_over(&mut self, index: usize) {
        let button = self.hand_overs().remove(index);
        let handler = button.handler.expect("the button acts");
        self.tab.ui(handler, None);
        self.settle();
    }

    /// The rows of every list on the page: title, second line.
    pub fn rows(&self) -> Vec<(String, Option<String>)> {
        let mut rows = Vec::new();
        self.tab.view().root.for_each(&mut |element| {
            if let Element::List(list) = element {
                rows.extend(
                    list.items
                        .iter()
                        .map(|item| (item.title.clone(), item.sub.clone())),
                );
            }
        });
        rows
    }

    /// The last `tab.setBadge` call's params.
    pub fn badge(&self) -> Option<Value> {
        self.fake.calls_to("tab.setBadge").pop()
    }

    /// The states of every `tab.setBadge` call, in order.
    pub fn badge_states(&self) -> Vec<Value> {
        self.fake
            .calls_to("tab.setBadge")
            .iter()
            .map(|params| params["state"].clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Button {
    pub label: String,
    pub kind: ButtonKind,
    pub handler: Option<HandlerId>,
}

impl Button {
    pub fn disabled(&self) -> bool {
        self.handler.is_none()
    }
}
