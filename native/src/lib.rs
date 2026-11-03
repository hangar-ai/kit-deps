//! The Dependencies tab of the deps kit, in Rust, built to WebAssembly: what
//! `tabs/deps/` (app.js, controller.js, view.js) does in a page, as a Hangar
//! native tab.
//!
//! Opening the tab reads the last audit from the kit's data folder and calls
//! nothing else: the audit command runs when the person clicks, and only then.
//! A finding is handed to an agent of the workspace on a click too.
//!
//! - `logic.rs`: the audit's document, checked, and the brief an agent gets
//!   (ported from logic.js, nothing of Hangar in it);
//! - `strings.rs`: the words, in English and French;
//! - `view.rs`: the page, drawn from the state;
//! - this file: the state, the messages, the calls.
//!
//! Every refusal is a state of the page, drawn: `denied` says where the
//! permission is given, a failed run says why.

mod logic;
mod strings;
mod view;

pub use logic::*;
pub use strings::{EN, FR, is_french, strings};

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use hangar_kit_sdk::api::{BadgeState, CommandAnswer, DataText, Done, Pane, Queued, Theme};
use hangar_kit_sdk::i18n::Strings;
use hangar_kit_sdk::prelude::*;

/// How often "Last run 5 min ago" is brought up to date, while the tab is seen.
pub const TICK: Duration = Duration::from_secs(30);

/// Which of the page's three shapes is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The last audit and the agents are being read.
    Loading,
    Ready,
    /// `command:audit` is not granted.
    Denied,
}

/// Why a run did not complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunFailure {
    Timeout,
    NotFound,
    Busy,
    /// The script printed something that is not an audit.
    Unreadable,
    Failed,
}

impl RunFailure {
    /// The key of its message.
    fn key(self) -> &'static str {
        match self {
            RunFailure::Timeout => "error.timeout",
            RunFailure::NotFound => "error.not_found",
            RunFailure::Busy => "error.busy",
            RunFailure::Unreadable => "error.unreadable",
            RunFailure::Failed => "error.failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunError {
    pub kind: RunFailure,
    pub detail: String,
}

/// Why a hand-over was refused, for the finding it was about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// `denied`, `not_found`, `rate_limited`, `unavailable` or `failed`.
    pub code: &'static str,
    pub detail: String,
    pub agent: String,
}

pub struct Deps {
    pub t: Strings,
    pub screen: Screen,
    pub running: bool,
    /// The last audit, checked.
    pub result: Option<AuditResult>,
    /// Set when a run did not complete; the last audit stays.
    pub error: Option<RunError>,
    pub panes: Vec<Pane>,
    pub panes_denied: bool,
    /// The agent the person picked.
    pub choice: Option<String>,
    /// Finding keys being sent.
    pub pending: BTreeSet<String>,
    /// Finding key → the agent it was handed to.
    pub sent: BTreeMap<String, String>,
    pub problems: BTreeMap<String, Problem>,
    /// List → rows drawn past the first few: a press of "Show N more" adds a
    /// step. The lists are the vulnerabilities and each gap of the outdated.
    pub more: BTreeMap<String, usize>,
    visible: bool,
    now: i64,
    utc_offset: i32,
    last_read: bool,
    panes_read: bool,
    /// The badge last sent, so that the same one is not sent twice.
    last_badge: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Last(Result<DataText, KitError>),
    Panes(Result<Vec<Pane>, KitError>),
    PanesChanged,
    Run,
    Ran(Result<CommandAnswer, KitError>),
    Saved(Result<Done, KitError>),
    Choose(String),
    ShowMore(String),
    HandOver(FindingKind, String),
    Sent {
        key: String,
        agent: String,
        answer: Result<Queued, KitError>,
    },
    Tick,
    Visible(bool),
    Theme(Theme),
}

impl Deps {
    /// The agent a hand-over goes to: the one picked, else the default.
    pub fn target(&self) -> Option<String> {
        self.choice.clone().or_else(|| default_target(&self.panes))
    }

    fn badge_now(&self) -> Option<Badge> {
        if self.running {
            Some(Badge::Running)
        } else {
            badge_for(self.result.as_ref())
        }
    }

    /// Tells the tab strip the badge, unless it already has it.
    fn set_badge(&mut self, cx: &mut Context<Message>) {
        let (state, text) = match self.badge_now() {
            None => (None, None),
            Some(Badge::Running) => (Some(BadgeState::Running), None),
            Some(Badge::Failure(high)) => (
                Some(BadgeState::Failure),
                Some(self.t.tn("badgeHigh", high as i64, &[])),
            ),
            Some(Badge::Neutral) => (Some(BadgeState::Neutral), None),
            Some(Badge::Ok) => (Some(BadgeState::Success), Some(self.t.t("badgeOk"))),
        };
        let id = format!("{state:?}|{text:?}");
        if self.last_badge.as_ref() == Some(&id) {
            return;
        }
        self.last_badge = Some(id);
        cx.call_quietly(api::TabSetBadge { state, text });
    }

    fn set_title(&self, cx: &mut Context<Message>) {
        cx.call_quietly(api::TabSetTitle {
            title: self.t.t("tab"),
        });
    }

    /// Ready once the last audit and the agents have both been read.
    fn ready_when_read(&mut self, cx: &mut Context<Message>) {
        if self.screen == Screen::Loading && self.last_read && self.panes_read {
            self.screen = Screen::Ready;
            self.set_badge(cx);
        }
    }

    fn fail(&mut self, kind: RunFailure, detail: &str) {
        self.error = Some(RunError {
            kind,
            detail: detail.chars().take(200).collect(),
        });
    }

    /// A finished command: the new audit, or why there is none.
    fn ran(&mut self, answer: Result<CommandAnswer, KitError>, cx: &mut Context<Message>) {
        match answer {
            Err(error) => match error.code.as_str() {
                KitError::DENIED => self.screen = Screen::Denied,
                KitError::NOT_FOUND => self.fail(RunFailure::NotFound, ""),
                KitError::BUSY => self.fail(RunFailure::Busy, ""),
                _ => self.fail(RunFailure::Failed, &error.message),
            },
            Ok(answer) if answer.timed_out => self.fail(RunFailure::Timeout, ""),
            Ok(answer) => {
                let clean = answer.exit_code == Some(0);
                match clean.then(|| parse_result(&answer.stdout)).flatten() {
                    Some(result) => {
                        // What the person sees is the new audit's: nothing of
                        // the last one's hand-overs or rows opened stays.
                        cx.call(
                            api::DataWrite {
                                path: LAST.to_owned(),
                                text: result.document().to_string(),
                            },
                            Message::Saved,
                        );
                        self.result = Some(result);
                        self.sent.clear();
                        self.problems.clear();
                        self.more.clear();
                    }
                    None if clean => self.fail(RunFailure::Unreadable, ""),
                    None => {
                        let detail = answer
                            .stderr
                            .lines()
                            .find(|line| !line.is_empty())
                            .map(str::to_owned)
                            .unwrap_or_else(|| match answer.exit_code {
                                Some(code) => self.t.tv("exitCode", &[("code", &code.to_string())]),
                                None => self.t.t("noExit"),
                            });
                        self.fail(RunFailure::Failed, &detail);
                    }
                }
            }
        }
        self.running = false;
        self.set_badge(cx);
    }

    /// A hand-over answered: handed, or the reason it was not.
    fn handed(
        &mut self,
        key: String,
        agent: String,
        answer: Result<Queued, KitError>,
        cx: &mut Context<Message>,
    ) {
        self.pending.remove(&key);
        match answer {
            Ok(_) => {
                self.sent.insert(key, agent);
            }
            Err(error) => {
                let code = match error.code.as_str() {
                    KitError::DENIED => "denied",
                    KitError::NOT_FOUND => "not_found",
                    KitError::RATE_LIMITED => "rate_limited",
                    KitError::UNAVAILABLE => "unavailable",
                    _ => "failed",
                };
                if matches!(code, "not_found" | "unavailable") {
                    // The agent is gone or busy: the list of agents is stale.
                    cx.call(api::PanesList {}, Message::Panes);
                }
                self.problems.insert(
                    key,
                    Problem {
                        code,
                        detail: error.message,
                        agent,
                    },
                );
            }
        }
    }
}

impl Kit for Deps {
    type Message = Message;

    fn init(start: &Start, cx: &mut Context<Message>) -> Self {
        let deps = Deps {
            t: strings(&start.theme.locale),
            screen: Screen::Loading,
            running: false,
            result: None,
            error: None,
            panes: Vec::new(),
            panes_denied: false,
            choice: None,
            pending: BTreeSet::new(),
            sent: BTreeMap::new(),
            problems: BTreeMap::new(),
            more: BTreeMap::new(),
            visible: start.visible,
            now: cx.now(),
            utc_offset: start.utc_offset,
            last_read: false,
            panes_read: false,
            last_badge: None,
        };
        // Nothing here runs the audit: opening the tab makes no network call.
        deps.set_title(cx);
        cx.call(
            api::DataRead {
                path: LAST.to_owned(),
            },
            Message::Last,
        );
        cx.call(api::PanesList {}, Message::Panes);
        deps
    }

    fn update(&mut self, message: Message, cx: &mut Context<Message>) {
        self.now = cx.now();
        match message {
            Message::Last(answer) => {
                // Nothing run yet, or a data folder that is not ours to read:
                // the page still works.
                self.result = answer.ok().and_then(|read| parse_result(&read.text));
                self.last_read = true;
                self.ready_when_read(cx);
            }
            Message::Panes(answer) => {
                match answer {
                    Ok(panes) => {
                        self.panes = panes;
                        self.panes_denied = false;
                    }
                    Err(error) => {
                        self.panes = Vec::new();
                        self.panes_denied = error.is(KitError::DENIED);
                    }
                }
                self.panes_read = true;
                self.ready_when_read(cx);
            }
            Message::PanesChanged => cx.call(api::PanesList {}, Message::Panes),
            // On the person's click.
            Message::Run => {
                if self.running {
                    return;
                }
                self.running = true;
                self.error = None;
                if self.screen == Screen::Denied {
                    self.screen = Screen::Ready;
                }
                self.set_badge(cx);
                cx.call(
                    api::CommandRun {
                        name: "audit".to_owned(),
                        args: BTreeMap::new(),
                    },
                    Message::Ran,
                );
            }
            Message::Ran(answer) => self.ran(answer, cx),
            Message::Saved(Ok(_)) => {}
            // It shows now; the next opening starts without it.
            Message::Saved(Err(_)) => cx.toast(Notice::Warning, self.t.t("notSaved")),
            Message::Choose(agent) => self.choice = Some(agent),
            Message::ShowMore(list) => *self.more.entry(list).or_insert(0) += SHOW_MORE_STEP,
            Message::HandOver(kind, key) => {
                let Some(agent) = self.target() else { return };
                let Some(finding) = self
                    .result
                    .as_ref()
                    .and_then(|result| find_finding(result, kind, &key))
                else {
                    return;
                };
                if self.pending.contains(&key) {
                    return;
                }
                self.pending.insert(key.clone());
                self.problems.remove(&key);
                cx.call(
                    api::PanesSend {
                        pane: agent.clone(),
                        text: finding.brief(),
                    },
                    move |answer| Message::Sent { key, agent, answer },
                );
            }
            Message::Sent { key, agent, answer } => self.handed(key, agent, answer, cx),
            Message::Tick => {}
            Message::Visible(visible) => self.visible = visible,
            // Hangar's language changed: the strip's title and badge follow.
            Message::Theme(theme) => {
                let changed = is_french(self.t.locale()) != is_french(&theme.locale);
                self.t.set_locale(theme.locale);
                if changed {
                    self.set_title(cx);
                    self.last_badge = None;
                    if self.screen != Screen::Loading {
                        self.set_badge(cx);
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        self.page()
    }

    fn subscriptions(&self) -> Subscriptions<Message> {
        let mut subscriptions = Subscriptions::none()
            .panes_changed(Message::PanesChanged)
            .visibility(Message::Visible)
            .theme(Message::Theme);
        // The clock only brings "Last run 5 min ago" up to date: behind
        // another tab nobody reads it.
        if self.visible {
            subscriptions = subscriptions.every("clock", TICK, Message::Tick);
        }
        subscriptions
    }
}

export_kit!(Deps);
