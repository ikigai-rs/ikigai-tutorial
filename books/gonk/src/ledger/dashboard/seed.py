#!/usr/bin/env python3
"""Fake history for an invented project, Pinwheel, as the commands that write it.

The gonk Book's chapter "The Dashboard" and `scripts/capture-dashboard.sh` both run this over a
scratch gonk. It prints one `ikigai` command per line, for one `ikigai` to read on standard input:

    python3 seed.py | ikigai --connect quic://127.0.0.1:1070 --cert-dir <importer's bundle>

Nothing here is random and no name is real, so every count the chapter prints comes from this file.
Every date is an offset from when it runs, because the Dashboard's windows ("Completed, 24 hours")
count back from the clock: a seed of fixed dates would drift out of them. So the counts never move,
and only the dates on the pages do.

The dates are real in the graph, not faked in a page. The seed's client is an import principal
(`ikigai-gonk ledger enroll --importer`), the one caller the ledger lets name a past date, and the
ledger marks each date it claims. See the chapter for which write takes which date.
"""

from datetime import datetime, timedelta, timezone

# On the minute, so no offset lands after the moment the ledger reads as now.
NOW = datetime.now(timezone.utc).replace(second=0, microsecond=0)

MINUTE = 1
HOUR = 60 * MINUTE
DAY = 24 * HOUR

# Two machines, each running a Claude loop and a Codex loop, as "Routing work to agents" sets up.
ALPHA_CLAUDE, ALPHA_CODEX = "alpha-claude", "alpha-codex"
BETA_CLAUDE, BETA_CODEX = "beta-claude", "beta-codex"

# How long each holder takes, claim to hand-back, in minutes: the Agents tab's median comes out of
# these, one per pass over the list. Codex's loops are quicker here. That is this story, not a claim.
PACE = {
    ALPHA_CLAUDE: [95, 140, 70, 210, 120],
    ALPHA_CODEX: [40, 65, 30, 85, 55],
    BETA_CLAUDE: [150, 110, 260, 130, 180],
    BETA_CODEX: [50, 35, 75, 45, 60],
}

# The dispatcher is away for these days ago: what is handed back then waits, and is closed in a batch
# on their return, so the chapter's open line climbs and falls.
AWAY = (29 * DAY, 38 * DAY)

# What the loops do with an item, in the order the ledger's items cycle through them:
#
#   done      claimed, handed back, closed done
#   rejected  claimed, handed back, sent back with a reason, handed back again, closed done
#   moved     claimed by another holder and plainly released, then as `done` under this holder
#   stopped   claimed, its run stopped, released with no hand-back; it stays open
#   declined  claimed, handed back, closed as not worth doing, so no completion is counted
#   reopened  as `done`, then reopened and closed again
STORY = ["done", "done", "moved", "done", "rejected", "done", "declined", "done", "stopped",
         "reopened", "done"]

# A ledger's filing history, oldest first: (days ago, title). `holders` take its items in turn, and
# a holder named twice takes twice the share, so the numbers differ per holder and per ledger.
LEDGERS = {
    "app": {
        "holders": [ALPHA_CLAUDE, BETA_CODEX, ALPHA_CODEX, ALPHA_CLAUDE, BETA_CLAUDE, BETA_CODEX],
        "nouns": ["the spinner", "the color wheel", "the share sheet", "the sign-in page",
                  "the settings screen", "the offline banner", "the undo stack", "the export dialog",
                  "the onboarding tour", "the search box", "the sync badge"],
        "verbs": ["Fix", "Tidy", "Test", "Speed up", "Label"],
        "days": [56, 55, 55, 54, 53, 52, 52, 51, 49, 49, 48, 47, 46, 46, 45, 44, 42, 42, 41, 40, 39,
                 39, 38, 36, 35, 35, 34, 33, 32, 32, 31, 30, 28, 28, 27, 26, 25, 25, 24, 22, 21, 21,
                 20, 19, 18, 18, 17, 16, 15, 14, 14, 13, 12, 11, 11, 10, 9, 9, 8, 7, 7, 6, 5, 5, 4],
    },
    "docs": {
        "holders": [BETA_CODEX, ALPHA_CODEX, BETA_CODEX, BETA_CLAUDE],
        "nouns": ["the quick start", "the glossary", "the install page", "the release notes",
                  "the style guide", "the FAQ"],
        "verbs": ["Rewrite", "Proofread", "Shorten"],
        "days": [50, 47, 44, 44, 40, 36, 33, 33, 29, 27, 24, 21, 21, 18, 15, 14, 11, 9, 7, 6, 4],
    },
    "infra": {
        "holders": [ALPHA_CLAUDE, BETA_CLAUDE, BETA_CLAUDE, ALPHA_CODEX],
        "nouns": ["the nightly build", "the backup timer", "the deploy script", "the disk alert",
                  "the certificate renewal", "the build cache", "the mirror job"],
        "verbs": ["Repair", "Document", "Watch"],
        "days": [54, 52, 49, 47, 43, 43, 39, 37, 35, 31, 29, 27, 23, 23, 19, 16, 13, 12, 8, 6, 5],
    },
}

# The last days by hand, so the windows have something in them whenever this runs. Every offset
# is at least an hour from a window's edge, so a run a minute later counts the same.
#
# (ledger, title, holder, filed, claimed, handed back, closed), minutes ago, None for "not yet".
# A row with no close is open now, and its claim is live, so it is what "In flight" counts.
RECENT = [
    ("app", "Fix the color wheel on small screens", ALPHA_CLAUDE,
     3 * DAY, 3 * DAY - 2 * HOUR, 3 * DAY - 4 * HOUR, 2 * DAY),
    ("app", "Test the share sheet offline", BETA_CODEX,
     2 * DAY, 2 * DAY - HOUR, 2 * DAY - 2 * HOUR, 30 * HOUR),
    ("app", "Label the undo stack's buttons", ALPHA_CODEX,
     30 * HOUR, 28 * HOUR, 27 * HOUR, 20 * HOUR),
    ("app", "Speed up the search box", ALPHA_CLAUDE,
     20 * HOUR, 18 * HOUR, 16 * HOUR, 6 * HOUR),
    ("app", "Tidy the sign-in page", BETA_CODEX,
     12 * HOUR, 11 * HOUR, 10 * HOUR, 3 * HOUR),
    ("app", "Fix the sync badge after a retry", BETA_CLAUDE,
     10 * HOUR, 9 * HOUR, 6 * HOUR, None),
    ("app", "Test the export dialog with long names", ALPHA_CLAUDE,
     5 * HOUR, 4 * HOUR, None, None),
    ("app", "Fix the offline banner's wording", BETA_CODEX,
     4 * HOUR, 2 * HOUR, None, None),
    ("docs", "Proofread the release notes", BETA_CODEX,
     2 * DAY, 2 * DAY - HOUR, 2 * DAY - 2 * HOUR, 40 * HOUR),
    ("docs", "Shorten the quick start", ALPHA_CODEX,
     18 * HOUR, 17 * HOUR, 16 * HOUR, 8 * HOUR),
    ("docs", "Rewrite the FAQ's first answer", BETA_CODEX,
     6 * HOUR, 5 * HOUR, None, None),
    ("infra", "Repair the mirror job's retry", BETA_CLAUDE,
     3 * DAY, 3 * DAY - HOUR, 3 * DAY - 4 * HOUR, 2 * DAY),
    ("infra", "Watch the disk alert for a week", ALPHA_CLAUDE,
     26 * HOUR, 25 * HOUR, 23 * HOUR, 12 * HOUR),
    ("infra", "Document the backup timer", ALPHA_CODEX,
     9 * HOUR, 8 * HOUR, 7 * HOUR, None),
]

# Filed and not yet taken: (ledger, title, minutes ago, routing labels). No `worker:` label is the
# shared pool's, which is how "Routing work to agents" reads a dispatch that names no machine.
WAITING = [
    ("app", "Fix the settings screen's back button", 7 * HOUR, "worker:pool"),
    ("app", "Test the onboarding tour on a tablet", 3 * HOUR, "worker:alpha,agent:claude"),
    ("docs", "Rewrite the style guide's opening", 8 * HOUR, "worker:pool"),
    ("infra", "Repair the certificate renewal", 5 * HOUR, "worker:beta,agent:codex"),
]

# Each machine's status item, the one a worker finds by its label and posts its lines on.
STATUS = [
    ("alpha", "Worker: alpha, loops alpha-claude and alpha-codex", "Disk: alpha 212 GiB free"),
    ("beta", "Worker: beta, loops beta-claude and beta-codex", "Disk: beta 41 GiB free"),
]


def routing(holder, turn):
    """Every third item is the shared pool's, so either machine takes it."""
    if turn % 3 == 2:
        return "worker:pool"
    machine, agent = holder.split("-")
    return f"worker:{machine},agent:{agent}"


class Session:
    """The commands, written as they would be typed. `ago` is minutes before the run."""

    def __init__(self, ledger):
        self.ledger = ledger
        self.number = 0
        self.events = 0

    def line(self, text):
        print(text)

    def at(self, ago):
        return (NOW - timedelta(minutes=ago)).strftime("%Y-%m-%dT%H:%M:%SZ")

    def sink(self, action, *args, content):
        words = " ".join(args)
        self.line(f"sink urn:iki:ledger:{self.ledger}:{action} {words} {content}")

    def file(self, title, ago, labels):
        self.number += 1
        self.sink("append", f"labels={labels}", f"created={self.at(ago)}", content=title)
        return self.number

    def history(self, item, *rows):
        """Past-dated claims and releases: history only, so the item itself does not change."""
        turtle = ["@prefix ledger: <https://ikigai-rs.dev/ns/ledger#> .",
                  "@prefix dcterms: <http://purl.org/dc/terms/> .",
                  "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> ."]
        for kind, holder, ago, *reason in rows:
            self.events += 1
            event = f"<urn:pinwheel:{self.ledger}:event:{self.events}>"
            terms = f'ledger:claimedBy "{holder}"'
            if reason:
                terms += f' ; ledger:releaseReason "{reason[0]}"'
            turtle.append(f'{event} a ledger:{kind} ; dcterms:created "{self.at(ago)}"^^xsd:dateTime ; '
                          f"{terms} .")
        self.sink("history", f"item={item}", content=" ".join(turtle))

    def claim(self, item, holder):
        """A hand-back names whoever holds the item, so this one is live, and dated now."""
        self.line(f"sink urn:iki:ledger:{self.ledger}:claim item={item} lease=720h {holder}")

    def say(self, item, ago, text, *flags):
        self.sink("comment", f"item={item}", f"created={self.at(ago)}", *flags, content=text)

    def close(self, item, ago, reason="done"):
        self.sink("close", f"item={item}", f"reason={reason}", f"closed-at={self.at(ago)}",
                  content="Closed.")

    def reopen(self, item, ago):
        self.sink("reopen", f"item={item}", f"created={self.at(ago)}", content="Reopened.")

    def handed_back_item(self, title, holder, labels, filed, claimed, back):
        item = self.file(title, filed, labels)
        self.claim(item, holder)
        self.history(item, ("ClaimEvent", holder, claimed))
        self.say(item, back, "Handed back: the change is ready for review.", "handback=true")
        return item

    def told(self, kind, title, holder, other, labels, filed, took):
        """One item from the older weeks, as `kind` says, all of it over before the recent rows."""
        claimed = filed - 3 * HOUR
        if kind == "stopped":
            item = self.file(title, filed, labels)
            self.history(item, ("ClaimEvent", holder, claimed),
                         ("ReleaseEvent", holder, claimed - 20 * MINUTE, "released"))
            self.say(item, claimed - 20 * MINUTE, "Run stopped: the machine went down for updates.")
            return
        if kind == "moved":
            item = self.file(title, filed, labels)
            let_go = claimed - 45 * MINUTE
            self.history(item, ("ClaimEvent", other, claimed),
                         ("ReleaseEvent", other, let_go, "released"))
            claimed = let_go - HOUR
            back = claimed - took
            self.claim(item, holder)
            self.history(item, ("ClaimEvent", holder, claimed))
            self.say(item, back, "Handed back: the change is ready for review.", "handback=true")
        else:
            back = claimed - took
            item = self.handed_back_item(title, holder, labels, filed, claimed, back)
        if kind == "rejected":
            self.say(item, back - 4 * HOUR, "Not yet: the test still fails on a small screen.",
                     "rejects=true")
            back -= 9 * HOUR
            self.say(item, back, "Handed back again: the test passes now.", "handback=true")
        closed = back - 5 * HOUR
        if AWAY[0] < closed < AWAY[1]:
            closed = AWAY[0] - (self.number % 40) * MINUTE
        if kind == "declined":
            self.close(item, closed, "wontfix")
            return
        self.close(item, closed)
        if kind == "reopened":
            self.reopen(item, closed - 21 * HOUR)
            self.close(item, closed - 25 * HOUR)


def main():
    other_of = {ALPHA_CLAUDE: BETA_CLAUDE, ALPHA_CODEX: BETA_CODEX,
                BETA_CLAUDE: ALPHA_CLAUDE, BETA_CODEX: ALPHA_CODEX}
    for name, ledger in LEDGERS.items():
        session = Session(name)
        passes = {holder: 0 for holder in PACE}
        for turn, days in enumerate(ledger["days"]):
            holder = ledger["holders"][turn % len(ledger["holders"])]
            verb = ledger["verbs"][turn % len(ledger["verbs"])]
            noun = ledger["nouns"][turn % len(ledger["nouns"])]
            pace = PACE[holder]
            took = pace[passes[holder] % len(pace)]
            passes[holder] += 1
            filed = days * DAY - (turn % 5) * 2 * HOUR
            session.told(STORY[turn % len(STORY)], f"{verb} {noun}", holder, other_of[holder],
                         routing(holder, turn), filed, took)
        for ledger_name, title, holder, filed, claimed, back, closed in RECENT:
            if ledger_name != name:
                continue
            labels = routing(holder, session.number)
            if back is None:
                item = session.file(title, filed, labels)
                session.claim(item, holder)
                session.history(item, ("ClaimEvent", holder, claimed))
            else:
                item = session.handed_back_item(title, holder, labels, filed, claimed, back)
            if closed is not None:
                session.close(item, closed)
        for ledger_name, title, filed, labels in WAITING:
            if ledger_name == name:
                session.file(title, filed, labels)
        if name == "infra":
            for machine, worker, disk in STATUS:
                item = session.file(f"Status of {machine}", 60 * DAY, f"worker-status:{machine}")
                session.say(item, 90 * MINUTE, disk)
                session.say(item, 80 * MINUTE, worker)


if __name__ == "__main__":
    main()
