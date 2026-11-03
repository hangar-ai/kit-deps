//! The tab's words, in English and French: `STRINGS` of `tabs/deps/logic.js`,
//! in the shape `hangar_kit_sdk::i18n::Strings` reads (a count `n` picks
//! `key_one` or `key_other`). `now` and `ago` are gone: `format::since` says
//! how long ago, in both languages.

use hangar_kit_sdk::i18n::Strings;

pub const EN: &[(&str, &str)] = &[
    ("tab", "Dependencies"),
    ("run", "Run the audit"),
    ("runAgain", "Run again"),
    ("running", "Auditing…"),
    ("loading", "Reading the last audit…"),
    (
        "runningText",
        "Asking each package manager what is vulnerable or out of date. This reads the registry and can take a minute.",
    ),
    ("neverTitle", "Not audited yet"),
    (
        "neverText",
        "Run the audit to list the vulnerable and out-of-date packages of this project. Nothing is sent or changed: the tools only read, and only when you ask.",
    ),
    ("stateNever", "Not run yet"),
    ("stateHigh_one", "{n} high or critical"),
    ("stateHigh_other", "{n} high or critical"),
    ("stateVulns_one", "{n} vulnerability"),
    ("stateVulns_other", "{n} vulnerabilities"),
    ("stateClean", "No known vulnerability"),
    ("stateOutdatedOnly", "Nothing vulnerable"),
    ("stateIncomplete", "Audit incomplete"),
    ("stateToolMissing", "Tool missing"),
    ("stateNoLock", "No lockfile"),
    ("lastRun", "Last run {time}"),
    ("cleanTitle", "Nothing to fix"),
    (
        "cleanText",
        "No known vulnerability and no package behind its latest version.",
    ),
    ("noLockTitle", "No lockfile found"),
    (
        "noLockText",
        "The audit reads pnpm-lock.yaml, package-lock.json, yarn.lock, Cargo.lock or requirements.txt in the workspace folder, and found none. Install the dependencies once so the lockfile exists, then run it again.",
    ),
    ("toolMissingTitle", "The audit tool is not installed"),
    (
        "toolMissingText",
        "The project has a lockfile, but none of the tools that read it is on this computer. Install one, then run the audit again.",
    ),
    (
        "missingTool",
        "{tool} is not installed. Install it with {command}, then run the audit again.",
    ),
    (
        "missingToolPlain",
        "{tool} is not installed. Install it, then run the audit again.",
    ),
    ("stepFailed", "{manager}: the {step} check failed. {detail}"),
    (
        "stepUnsupported",
        "{manager}: the outdated check is not available for this version.",
    ),
    ("step.audit", "vulnerability"),
    ("step.outdated", "outdated"),
    (
        "truncated",
        "The lists are cut at 500 entries per package manager.",
    ),
    ("vulnerabilities", "Vulnerabilities"),
    ("outdated", "Outdated"),
    ("severity.critical", "Critical"),
    ("severity.high", "High"),
    ("severity.moderate", "Moderate"),
    ("severity.low", "Low"),
    ("severity.unknown", "Not graded"),
    ("kind.major", "Major"),
    ("kind.minor", "Minor"),
    ("kind.patch", "Patch"),
    ("fixedIn", "fixed in {version}"),
    ("noFix", "no fixed version listed"),
    ("devDependency", "dev"),
    ("showMore", "Show {n} more"),
    ("handOver", "Hand over to an agent"),
    ("handingOver", "Sending…"),
    ("sentTo", "Handed to {agent}"),
    ("agentLabel", "Agent"),
    ("noAgent", "No Claude Code agent can receive text now."),
    (
        "panesDenied",
        "This tab may not list the agents. Accept panes.read in Settings › Kits to hand a package over.",
    ),
    (
        "problem.denied",
        "This tab may not send to agents. Accept panes.send in Settings › Kits.",
    ),
    (
        "problem.not_found",
        "{agent} is not in this workspace any more.",
    ),
    (
        "problem.rate_limited",
        "Too many sends in a minute. Try again in a moment.",
    ),
    (
        "problem.unavailable",
        "{agent} cannot receive text now. Pick another agent, or try again.",
    ),
    ("problem.failed", "It could not be sent: {detail}"),
    ("deniedTitle", "This tab may not run the audit"),
    (
        "deniedText",
        "Accept command:audit in Settings › Kits, then try again. The tab only runs the audit script it declares, and only when you click.",
    ),
    ("errorTitle", "The audit did not complete"),
    (
        "error.timeout",
        "It took longer than the time allowed. Try again; a slow registry is the usual cause.",
    ),
    (
        "error.not_found",
        "Node.js was not found on this computer. Install it, then try again.",
    ),
    (
        "error.busy",
        "Hangar is already running the most commands it allows. Try again in a moment.",
    ),
    (
        "error.unreadable",
        "The script printed something that is not an audit. Reload the tab, then try again.",
    ),
    ("error.failed", "The script could not run: {detail}"),
    ("noExit", "it ended without an exit code"),
    ("exitCode", "exit code {code}"),
    (
        "notSaved",
        "This audit could not be kept for the next opening.",
    ),
    ("copyCommand", "Copy the command"),
    ("copied", "Copied"),
    ("retry", "Try again"),
    ("badgeHigh_one", "{n} high"),
    ("badgeHigh_other", "{n} high"),
    ("badgeOk", "OK"),
];

pub const FR: &[(&str, &str)] = &[
    ("tab", "Dépendances"),
    ("run", "Lancer l’audit"),
    ("runAgain", "Relancer"),
    ("running", "Audit en cours…"),
    ("loading", "Lecture du dernier audit…"),
    (
        "runningText",
        "Chaque gestionnaire de paquets est interrogé sur ce qui est vulnérable ou en retard. Il lit le registre, ce qui peut prendre une minute.",
    ),
    ("neverTitle", "Pas encore audité"),
    (
        "neverText",
        "Lancez l’audit pour lister les paquets vulnérables ou en retard de ce projet. Rien n’est envoyé ni modifié : les outils ne font que lire, et seulement quand vous le demandez.",
    ),
    ("stateNever", "Pas encore lancé"),
    ("stateHigh_one", "{n} grave ou critique"),
    ("stateHigh_other", "{n} graves ou critiques"),
    ("stateVulns_one", "{n} faille"),
    ("stateVulns_other", "{n} failles"),
    ("stateClean", "Aucune faille connue"),
    ("stateOutdatedOnly", "Aucune faille"),
    ("stateIncomplete", "Audit incomplet"),
    ("stateToolMissing", "Outil manquant"),
    ("stateNoLock", "Aucun fichier de verrouillage"),
    ("lastRun", "Dernier audit {time}"),
    ("cleanTitle", "Rien à corriger"),
    (
        "cleanText",
        "Aucune faille connue et aucun paquet en retard sur sa dernière version.",
    ),
    ("noLockTitle", "Aucun fichier de verrouillage trouvé"),
    (
        "noLockText",
        "L’audit lit pnpm-lock.yaml, package-lock.json, yarn.lock, Cargo.lock ou requirements.txt dans le dossier de l’espace de travail, et n’en a trouvé aucun. Installez les dépendances une fois pour créer ce fichier, puis relancez l’audit.",
    ),
    ("toolMissingTitle", "L’outil d’audit n’est pas installé"),
    (
        "toolMissingText",
        "Le projet a un fichier de verrouillage, mais aucun des outils qui le lisent n’est présent sur cet ordinateur. Installez-en un, puis relancez l’audit.",
    ),
    (
        "missingTool",
        "{tool} n’est pas installé. Installez-le avec {command}, puis relancez l’audit.",
    ),
    (
        "missingToolPlain",
        "{tool} n’est pas installé. Installez-le, puis relancez l’audit.",
    ),
    (
        "stepFailed",
        "{manager} : la vérification des {step} a échoué. {detail}",
    ),
    (
        "stepUnsupported",
        "{manager} : la vérification des paquets en retard n’est pas disponible pour cette version.",
    ),
    ("step.audit", "failles"),
    ("step.outdated", "paquets en retard"),
    (
        "truncated",
        "Les listes sont coupées à 500 entrées par gestionnaire de paquets.",
    ),
    ("vulnerabilities", "Failles"),
    ("outdated", "En retard"),
    ("severity.critical", "Critique"),
    ("severity.high", "Grave"),
    ("severity.moderate", "Modérée"),
    ("severity.low", "Faible"),
    ("severity.unknown", "Non classée"),
    ("kind.major", "Majeure"),
    ("kind.minor", "Mineure"),
    ("kind.patch", "Correctif"),
    ("fixedIn", "corrigée en {version}"),
    ("noFix", "aucune version corrigée indiquée"),
    ("devDependency", "dev"),
    ("showMore", "Afficher {n} de plus"),
    ("handOver", "Confier à un agent"),
    ("handingOver", "Envoi…"),
    ("sentTo", "Confié à {agent}"),
    ("agentLabel", "Agent"),
    (
        "noAgent",
        "Aucun agent Claude Code ne peut recevoir de texte pour l’instant.",
    ),
    (
        "panesDenied",
        "Cet onglet n’a pas le droit de lister les agents. Acceptez panes.read dans Réglages › Kits pour confier un paquet.",
    ),
    (
        "problem.denied",
        "Cet onglet n’a pas le droit d’écrire aux agents. Acceptez panes.send dans Réglages › Kits.",
    ),
    (
        "problem.not_found",
        "{agent} n’est plus dans cet espace de travail.",
    ),
    (
        "problem.rate_limited",
        "Trop d’envois en une minute. Réessayez dans un instant.",
    ),
    (
        "problem.unavailable",
        "{agent} ne peut pas recevoir de texte pour l’instant. Choisissez un autre agent, ou réessayez.",
    ),
    ("problem.failed", "L’envoi a échoué : {detail}"),
    (
        "deniedTitle",
        "Cet onglet n’a pas le droit de lancer l’audit",
    ),
    (
        "deniedText",
        "Acceptez command:audit dans Réglages › Kits, puis réessayez. L’onglet ne lance que le script d’audit qu’il déclare, et seulement quand vous cliquez.",
    ),
    ("errorTitle", "L’audit ne s’est pas terminé"),
    (
        "error.timeout",
        "Il a pris plus de temps que prévu. Réessayez ; un registre lent en est la cause habituelle.",
    ),
    (
        "error.not_found",
        "Node.js est introuvable sur cet ordinateur. Installez-le, puis réessayez.",
    ),
    (
        "error.busy",
        "Hangar lance déjà le maximum de commandes autorisé. Réessayez dans un instant.",
    ),
    (
        "error.unreadable",
        "Le script a affiché autre chose qu’un audit. Rechargez l’onglet, puis réessayez.",
    ),
    ("error.failed", "Le script n’a pas pu se lancer : {detail}"),
    ("noExit", "il s’est arrêté sans code de sortie"),
    ("exitCode", "code de sortie {code}"),
    (
        "notSaved",
        "Cet audit n’a pas pu être gardé pour la prochaine ouverture.",
    ),
    ("copyCommand", "Copier la commande"),
    ("copied", "Copié"),
    ("retry", "Réessayer"),
    ("badgeHigh_one", "{n} grave"),
    ("badgeHigh_other", "{n} graves"),
    ("badgeOk", "OK"),
];

/// The tab's words, spoken in `locale`.
pub fn strings(locale: &str) -> Strings {
    Strings::new(locale).add("en", EN).add("fr", FR)
}

/// Whether `locale` is French: `pickLang` of the page.
pub fn is_french(locale: &str) -> bool {
    locale
        .split(['-', '_'])
        .next()
        .is_some_and(|lang| lang.eq_ignore_ascii_case("fr"))
}
