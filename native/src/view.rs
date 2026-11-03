//! The page, drawn from the state (view.js): Hangar UI's widgets and no colour
//! of its own. An advisory's title is only ever text.

use hangar_kit_sdk::Handler;
use hangar_kit_sdk::format;
use hangar_kit_sdk::prelude::*;
use hangar_kit_sdk::ui::{Glyph, ListItem, SelectOption};

use crate::logic::{self, AuditStatus, FindingKind, Found, Outdated, Step, Vulnerability};
use crate::{Deps, Message, Screen};

type Row = ListItem<Handler<Message>>;

/// The parts of a line that are there, joined.
fn joined(parts: &[&str], with: &str) -> String {
    parts
        .iter()
        .filter(|part| !part.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(with)
}

impl Deps {
    pub(crate) fn page(&self) -> Element<Message> {
        let mut children = vec![self.head()];
        children.extend(match self.screen {
            Screen::Loading => vec![loading(self.t.t("loading")).into()],
            Screen::Denied => vec![
                denied(self.t.t("deniedTitle"))
                    .text(self.t.t("deniedText"))
                    .action(
                        button(self.t.t("retry"))
                            .small()
                            .primary()
                            .on_press(Message::Run),
                    )
                    .into(),
            ],
            Screen::Ready => self.ready(),
        });
        page(children).into()
    }

    /// The title, the state of the audit, when it last ran, and the button
    /// that runs it.
    fn head(&self) -> Element<Message> {
        let mut head = header(self.t.t("tab"));
        if self.screen != Screen::Loading {
            head = head.state(self.state_line());
            if let (Some(result), false) = (&self.result, self.running) {
                let time = format::since(result.at, self.now, self.utc_offset, self.t.locale());
                head = head.state(text(self.t.tv("lastRun", &[("time", &time)])).small().dim());
            }
        }
        if self.screen == Screen::Ready {
            let label = self.t.t(if self.result.is_some() {
                "runAgain"
            } else {
                "run"
            });
            let mut run = button(label)
                .small()
                .on_press_maybe((!self.running).then_some(Message::Run));
            if self.result.is_none() {
                run = run.primary();
            }
            head = head.action(run);
        }
        head.into()
    }

    /// One state, as a glyph and a word.
    fn state_line(&self) -> Element<Message> {
        let t = &self.t;
        let (status, label) = if self.running {
            (Status::Running, t.t("running"))
        } else if let Some(result) = &self.result {
            if logic::answered(result) {
                let vulns = logic::vulnerabilities_of(result);
                let high = logic::high_count(&vulns);
                if high > 0 {
                    (Status::Failure, t.tn("stateHigh", high as i64, &[]))
                } else if !vulns.is_empty() {
                    (Status::Neutral, t.tn("stateVulns", vulns.len() as i64, &[]))
                } else if !logic::vulnerability_check_complete(result) {
                    (Status::Neutral, t.t("stateIncomplete"))
                } else if logic::outdated_groups(result).is_empty() {
                    (Status::Success, t.t("stateClean"))
                } else {
                    (Status::Success, t.t("stateOutdatedOnly"))
                }
            } else if result.status == AuditStatus::NoLockfile {
                (Status::Neutral, t.t("stateNoLock"))
            } else if result.status == AuditStatus::ToolMissing {
                (Status::Neutral, t.t("stateToolMissing"))
            } else {
                (Status::Neutral, String::new())
            }
        } else {
            (Status::Neutral, t.t("stateNever"))
        };
        if label.is_empty() {
            return space().into();
        }
        state_label(status, label).into()
    }

    /// The body of a page that is ready: a run in progress, why the last one
    /// failed, and what the last audit found.
    fn ready(&self) -> Vec<Element<Message>> {
        let t = &self.t;
        let mut body: Vec<Element<Message>> = Vec::new();
        if self.running {
            body.push(progress(None).label(t.t("running")).into());
            body.push(banner(Notice::Info, t.t("runningText")).into());
        }
        if let Some(error) = &self.error {
            let mut failure = failed(t.t("errorTitle"))
                .text(t.tv(error.kind.key(), &[("detail", &error.detail)]));
            if !self.running {
                // One primary action on the page: the header's, when there is
                // no audit to run again.
                let mut retry = button(t.t("retry")).small().on_press(Message::Run);
                if self.result.is_some() {
                    retry = retry.primary();
                }
                failure = failure.action(retry);
            }
            body.push(failure.into());
        }
        let Some(result) = &self.result else {
            if !self.running && self.error.is_none() {
                body.push(empty(t.t("neverTitle")).text(t.t("neverText")).into());
            }
            return body;
        };
        match result.status {
            AuditStatus::NoLockfile => body.push(
                empty(t.t("noLockTitle"))
                    .icon(Icon::Search)
                    .text(t.t("noLockText"))
                    .into(),
            ),
            AuditStatus::ToolMissing => {
                body.push(
                    empty(t.t("toolMissingTitle"))
                        .icon(Icon::Alert)
                        .text(t.t("toolMissingText"))
                        .into(),
                );
                body.extend(self.notices(result));
            }
            AuditStatus::Ok => body.push(self.findings(result)),
        }
        body
    }

    /// What the audit found, or that it found nothing.
    fn findings(&self, result: &logic::AuditResult) -> Element<Message> {
        let t = &self.t;
        let mut stack: Vec<Element<Message>> = self.notices(result);
        let vulns = logic::vulnerabilities_of(result);
        let groups = logic::outdated_groups(result);
        let multiple = result.managers.len() > 1;
        let nothing_said = result
            .managers
            .iter()
            .all(|m| m.errors.is_empty() && m.missing.is_empty());
        if vulns.is_empty() && groups.is_empty() && logic::answered(result) && nothing_said {
            stack.push(
                empty(t.t("cleanTitle"))
                    .icon(Icon::Check)
                    .text(t.t("cleanText"))
                    .into(),
            );
        } else {
            if !vulns.is_empty() || !groups.is_empty() {
                stack.push(self.agent_picker());
            }
            if !vulns.is_empty() {
                let rows = self.rows("vulnerability", &vulns, |found| {
                    self.vulnerability_row(found, multiple)
                });
                stack.push(
                    section(t.t("vulnerabilities"), rows)
                        .count(vulns.len().to_string())
                        .into(),
                );
            }
            for group in &groups {
                let title = format!(
                    "{} · {}",
                    t.t("outdated"),
                    t.t(&format!("kind.{}", group.kind.as_str()))
                );
                let rows = self.rows(group.kind.as_str(), &group.items, |found| {
                    self.outdated_row(found, multiple)
                });
                stack.push(
                    section(title, rows)
                        .count(group.items.len().to_string())
                        .into(),
                );
            }
        }
        column(stack).spacing(12.0).into()
    }

    /// What the managers said that is not a list: a tool to install, a check
    /// that failed.
    fn notices(&self, result: &logic::AuditResult) -> Vec<Element<Message>> {
        let t = &self.t;
        let mut out: Vec<Element<Message>> = Vec::new();
        for manager in &result.managers {
            for tool in &manager.missing {
                out.push(match logic::install_command(tool) {
                    Some(command) => banner(
                        Notice::Warning,
                        t.tv("missingTool", &[("tool", tool), ("command", command)]),
                    )
                    .action(
                        icon_button(Icon::Copy, t.t("copyCommand"))
                            .on_press(Message::Copy(command.to_owned())),
                    )
                    .into(),
                    None => {
                        banner(Notice::Warning, t.tv("missingToolPlain", &[("tool", tool)])).into()
                    }
                });
            }
            for error in &manager.errors {
                let step = t.t(&format!("step.{}", error.step.as_str()));
                let line = t.tv(
                    "stepFailed",
                    &[
                        ("manager", manager.manager.as_str()),
                        ("step", &step),
                        ("detail", &error.detail),
                    ],
                );
                out.push(banner(Notice::Error, line).into());
            }
            if manager.steps.outdated == Step::Unsupported {
                out.push(
                    banner(
                        Notice::Info,
                        t.tv("stepUnsupported", &[("manager", manager.manager.as_str())]),
                    )
                    .into(),
                );
            }
            if manager.truncated {
                out.push(banner(Notice::Warning, t.t("truncated")).into());
            }
        }
        out
    }

    /// One line saying who the findings go to: a choice when there are
    /// several agents.
    fn agent_picker(&self) -> Element<Message> {
        let t = &self.t;
        if self.panes_denied {
            return text(t.t("panesDenied")).small().dim().into();
        }
        let agents = logic::candidates(&self.panes);
        match agents.as_slice() {
            [] => text(t.t("noAgent")).small().dim().into(),
            [only] => text(format!("{} : {}", t.t("agentLabel"), only.name))
                .small()
                .dim()
                .into(),
            _ => {
                let options = agents
                    .iter()
                    .map(|pane| SelectOption::new(pane.name.clone(), pane.name.clone()));
                select(options, self.target())
                    .label(t.t("agentLabel"))
                    .on_select(Message::Choose)
                    .into()
            }
        }
    }

    /// The rows of a list, the first few when it is long and the rest a
    /// button away, a hundred at a time.
    fn rows<T>(
        &self,
        list_name: &str,
        items: &[Found<T>],
        draw: impl Fn(&Found<T>) -> Row,
    ) -> Vec<Element<Message>> {
        let shown = (logic::SHOWN_PER_GROUP + self.more.get(list_name).copied().unwrap_or(0))
            .min(items.len());
        let mut out: Vec<Element<Message>> =
            vec![list(items[..shown].iter().map(draw)).group().into()];
        let hidden = items.len() - shown;
        if hidden > 0 {
            let n = hidden.min(logic::SHOW_MORE_STEP);
            out.push(
                row![
                    button(self.t.tv("showMore", &[("n", &n.to_string())]))
                        .small()
                        .ghost()
                        .on_press(Message::ShowMore(list_name.to_owned()))
                ]
                .into(),
            );
        }
        out
    }

    fn vulnerability_row(&self, found: &Found<Vulnerability>, multiple: bool) -> Row {
        let t = &self.t;
        let v = &found.item;
        let installed = v.installed.as_deref();
        let versions = match &v.fixed {
            Some(fixed) => format!("{} → {fixed}", installed.unwrap_or("?")),
            None => installed.unwrap_or("").to_owned(),
        };
        let title = joined(&[&v.package, &versions], " ");
        let severity = t.t(&format!("severity.{}", v.severity.as_str()));
        let sub = joined(&[&severity, &v.title], " · ");
        let fix = match &v.fixed {
            Some(version) => t.tv("fixedIn", &[("version", version)]),
            None => t.t("noFix"),
        };
        let manager = if multiple { found.manager.as_str() } else { "" };
        let meta = joined(&[&v.id, &fix, manager], " · ");
        let status = if v.severity.is_bad() {
            Status::Failure
        } else {
            Status::Neutral
        };
        self.finding_row(
            FindingKind::Vulnerability,
            &found.key,
            glyph(status, severity),
            title,
            Some(sub),
            meta,
        )
    }

    fn outdated_row(&self, found: &Found<Outdated>, multiple: bool) -> Row {
        let t = &self.t;
        let o = &found.item;
        let title = format!(
            "{} {} → {}",
            o.package,
            o.current.as_deref().unwrap_or("?"),
            o.latest.as_deref().unwrap_or("?")
        );
        let gap = t.t(&format!("kind.{}", o.kind.as_str()));
        let dev = if o.dev {
            t.t("devDependency")
        } else {
            String::new()
        };
        let manager = if multiple { found.manager.as_str() } else { "" };
        let meta = joined(&[&gap, &dev, manager], " · ");
        self.finding_row(
            FindingKind::Outdated,
            &found.key,
            glyph(Status::Neutral, gap),
            title,
            None,
            meta,
        )
    }

    /// A finding: what it is, a line about it, why it was not handed over if
    /// it was refused, and the button that hands it to an agent.
    fn finding_row(
        &self,
        kind: FindingKind,
        key: &str,
        lead: Glyph,
        title: String,
        sub: Option<String>,
        meta: String,
    ) -> Row {
        let mut entry = item(title).lead(lead).wrap();
        if let Some(sub) = sub.filter(|sub| !sub.is_empty()) {
            entry = entry.sub(sub);
        }
        if !meta.is_empty() {
            entry = entry.body(text(meta).small().dim());
        }
        if let Some(problem) = self.problems.get(key) {
            let line = self.t.tv(
                &format!("problem.{}", problem.code),
                &[("agent", &problem.agent), ("detail", &problem.detail)],
            );
            entry = entry.body(banner(Notice::Error, line));
        }
        entry.body(row![self.hand_over(kind, key)])
    }

    /// The button that hands one finding to the chosen agent, or what stands
    /// in for it.
    fn hand_over(&self, kind: FindingKind, key: &str) -> Element<Message> {
        if let Some(agent) = self.sent.get(key) {
            return state_label(Status::Success, self.t.tv("sentTo", &[("agent", agent)])).into();
        }
        let pending = self.pending.contains(key);
        let label = self.t.t(if pending { "handingOver" } else { "handOver" });
        let can = !pending && self.target().is_some();
        button(label)
            .small()
            .on_press_maybe(can.then(|| Message::HandOver(kind, key.to_owned())))
            .into()
    }
}
