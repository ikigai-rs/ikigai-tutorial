#!/usr/bin/env bash
# Run "Tic-tac-toe, VI" (books/ikigai/src/applied/tic-tac-toe-6.md) for real, and check
# the page against what happens.
#
# The chapter's transcripts cannot run in the page: they need a Python process, a Deno
# process, ttt-host and two apps talking over sockets. So this script does what a reader
# would do, from the chapter's own text:
#
#   * clones ikigai-python and ikigai-deno at `main` into a scratch directory (it never
#     touches your own checkouts), and lays them out beside this repository the way the
#     chapter assumes, so `cd ../ikigai-python` in a command means that clone;
#   * builds ttt-host from this checkout and puts it first on PATH;
#   * checks every code excerpt the chapter quotes from those repositories
#     (`<!-- excerpt: <repo> <path> @ <commit> -->` above a fence) still appears in that
#     file, verbatim, at their `main`;
#   * walks the chapter in page order: a `<!-- demo: start -->` block's commands are run
#     (a line ending in `&` in the background, its output kept in a log); a
#     `<!-- demo: run -->` block's `$ ` commands are run and their output compared with the
#     lines under them; `<!-- demo: run same-as-previous -->` also requires the output to
#     equal the previous block's with the peer's thread name set aside; `<!-- demo: log -->`
#     lines must appear in the background processes' logs;
#   * stops everything it started.
#
# Two kinds of noise are normalized on both sides: durations (`0ms`) and Python's numbered
# thread names (`Thread-2`). A line that is exactly `…` in a block means "any lines here".
#
#   ./scripts/ttt-polyglot-demo.sh
#
# NOT RUN BY CI, and the chapter says so: it needs python3, deno, the ikigai CLI, curl,
# git, network access for the two clones, and ports 8070-8072 free. When python3 or deno
# (or another tool) is missing it says which and exits 0 — a skip, not a pass. It exits 1
# when anything differs.
#
# The sockets live under /tmp/ttt6demo, a short path on purpose: macOS allows a Unix socket
# path 104 bytes, and a session scratch directory is already most of that.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

missing=()
for tool in python3 deno ikigai curl git cargo shasum; do
    command -v "$tool" > /dev/null 2>&1 || missing+=("$tool")
done
if [ "${#missing[@]}" -gt 0 ]; then
    echo "ttt-polyglot-demo: SKIPPED — not installed: ${missing[*]}"
    echo "  (the chapter's transcripts need all of: python3 deno ikigai curl git cargo shasum)"
    exit 0
fi

cargo build --quiet -p ttt-host
export PATH="$PWD/target/debug:$PATH"

exec python3 - "$PWD" <<'PY'
import os, re, signal, socket, subprocess, sys, time, difflib
from pathlib import Path

REPO = Path(sys.argv[1])
CHAPTER = REPO / "books/ikigai/src/applied/tic-tac-toe-6.md"
DEMO = Path("/tmp/ttt6demo")
WORK = DEMO / "work"
PORTS = {8070: "ttt-host", 8071: "the Deno app", 8072: "the Python app"}
SIBLINGS = {
    "ikigai-python": os.environ.get("IKIGAI_PYTHON_URL", "https://github.com/ikigai-rs/ikigai-python"),
    "ikigai-deno": os.environ.get("IKIGAI_DENO_URL", "https://github.com/ikigai-rs/ikigai-deno"),
}

def listening(port):
    with socket.socket() as s:
        s.settimeout(0.2)
        return s.connect_ex(("127.0.0.1", port)) == 0

busy = [f"{port} ({who})" for port, who in PORTS.items() if listening(port)]
if busy:
    sys.exit(f"ttt-polyglot-demo: port {', '.join(busy)} is in use; stop it and run again")

subprocess.run(["rm", "-rf", str(DEMO)], check=True)
WORK.mkdir(parents=True)
(DEMO / "logs").mkdir()
os.symlink(REPO, WORK / "ikigai-tutorial")
for name, url in SIBLINGS.items():
    subprocess.run(["git", "clone", "--quiet", "--depth", "1", url, str(WORK / name)], check=True)
    head = subprocess.run(["git", "-C", str(WORK / name), "rev-parse", "--short", "HEAD"],
                          capture_output=True, text=True, check=True).stdout.strip()
    print(f"· {name} main is {head}")

text = CHAPTER.read_text()
failures = []

# -- the excerpts ----------------------------------------------------------------------
EXCERPT = re.compile(r"<!-- excerpt: (\S+) (\S+) @ ([0-9a-f]+) -->\n```\w*\n(.*?)\n```", re.S)
for repo, path, commit, body in EXCERPT.findall(text):
    source = (WORK / repo / path).read_text()
    where = f"excerpt from {repo} {path} (cited at {commit})"
    if body in source:
        print(f"✓ {where}: verbatim at main")
    else:
        failures.append(where)
        print(f"✗ {where}: NOT verbatim at main any more")

# -- the chapter, in page order ---------------------------------------------------------
DIRECTIVE = re.compile(r"<!-- demo: ([a-z -]+?) -->\n```\w*\n(.*?)\n```", re.S)
started = []

def normalize(out):
    out = re.sub(r"\b\d+ms\b", "Nms", out)
    return re.sub(r"Thread-\d+", "Thread-N", out)

def peerless(out):
    return re.sub(r" · (Thread-N \(_handle\)|deno-main) · ", " · PEER · ", out)

def matches(got, want):
    """`want`'s lines, with `…` meaning any lines, against `got`'s."""
    got = [l.rstrip() for l in got.rstrip("\n").split("\n")]
    segments, current = [], []
    for line in want:
        if line == "…":
            segments.append(current)
            current = []
        else:
            current.append(line.rstrip())
    segments.append(current)
    if len(segments) == 1:
        return got == segments[0]
    at = 0
    for i, segment in enumerate(segments):
        if not segment:
            continue
        if i == 0:
            if got[: len(segment)] != segment:
                return False
            at = len(segment)
            continue
        found = next((j for j in range(at, len(got) - len(segment) + 1)
                      if got[j : j + len(segment)] == segment), None)
        if found is None or (i == len(segments) - 1 and found + len(segment) != len(got)):
            return False
        at = found + len(segment)
    return True

def run(command):
    done = subprocess.run(["bash", "-c", f"cd {WORK}/ikigai-tutorial && {command}"],
                          capture_output=True, text=True, timeout=120)
    out = done.stdout + done.stderr
    return out if out.endswith("\n") or not out else out + "\n"

def wait_for(block):
    deadline = time.time() + 60
    sockets = set(re.findall(r"/tmp/ttt6demo/[\w.-]+\.sock", block))
    ports = set()
    if re.search(r"^ttt-host ", block, re.M):
        ports.add(8070)
    if "tictactoe_app.ts" in block:
        ports.add(8071)
    if "examples.tictactoe_app" in block:
        ports.add(8072)
    while time.time() < deadline:
        if all(Path(s).exists() for s in sockets) and all(listening(p) for p in ports):
            time.sleep(0.5)
            return
        time.sleep(0.2)
    raise SystemExit(f"ttt-polyglot-demo: never came up: {sorted(sockets)} {sorted(ports)}; "
                     f"see {DEMO}/logs")

previous = None
try:
    for kind, block in DIRECTIVE.findall(text):
        if kind == "start":
            for line in block.splitlines():
                line = line.strip()
                if not line:
                    continue
                if line.endswith("&"):
                    log = open(DEMO / "logs" / f"{len(started)}.log", "w")
                    started.append(subprocess.Popen(
                        ["bash", "-c", f"cd {WORK}/ikigai-tutorial && {line[:-1]}"],
                        stdout=log, stderr=subprocess.STDOUT, start_new_session=True))
                else:
                    subprocess.run(["bash", "-c", f"cd {WORK}/ikigai-tutorial && {line}"], check=True)
            wait_for(block)
            print(f"· started: {sum(1 for l in block.splitlines() if l.strip().endswith('&'))} processes")
            continue
        if kind == "log":
            logs = "".join(p.read_text() for p in sorted((DEMO / "logs").glob("*.log")))
            absent = [line for line in block.splitlines() if line not in logs]
            for line in absent:
                failures.append(f"log line: {line}")
                print(f"✗ not in any log: {line}")
            if not absent:
                print("✓ the startup lines are in the logs")
            continue
        # run / run same-as-previous
        commands, want, lines = [], [], block.splitlines()
        i = 0
        while i < len(lines):
            line = lines[i]
            if line.startswith("$ "):
                command = line[2:]
                while command.endswith("\\"):
                    i += 1
                    command = command[:-1] + " " + lines[i].strip()
                commands.append(command)
            else:
                want.append(line)
            i += 1
        got = ""
        for command in commands:
            got += run(command)
        got = normalize(got)
        want = [normalize(l) for l in want]
        first = commands[0] if commands else "?"
        label = f"{first[:70]}{'…' if len(first) > 70 else ''}"
        ok = matches(got, want)
        if kind == "run same-as-previous":
            same = previous is not None and peerless(got) == peerless(previous)
            if not same:
                print(f"✗ not the previous block's output, peer thread aside: {label}")
                sys.stdout.writelines(difflib.unified_diff(
                    peerless(previous or "").splitlines(True), peerless(got).splitlines(True),
                    "previous", "this"))
            ok = ok and same
        if ok:
            print(f"✓ {label}")
        else:
            failures.append(label)
            print(f"✗ {label}")
            sys.stdout.writelines(difflib.unified_diff(
                [l + "\n" for l in want], got.splitlines(True), "the chapter", "a real run"))
        previous = got
finally:
    for process in started:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass

if failures:
    print(f"\nttt-polyglot-demo: {len(failures)} difference(s) between the chapter and a real run")
    sys.exit(1)
print("\nttt-polyglot-demo: the chapter matches a real run")
PY
