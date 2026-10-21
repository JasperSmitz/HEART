#!/usr/bin/env python3
"""Exercise the real dashboard through an outer PTY; no model/provider required.

Uses temporary zsh rc/config and files, never rewrites the user's shell setup.
Run after `cargo build -p heart --locked` on Linux/macOS with zsh installed.
"""
import argparse
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import shlex
import shutil
import signal
import struct
import subprocess
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[1]
F12 = b"\x1b[24~"


class Dashboard:
    def __init__(self, binary, directory, shell_path=None):
        self.directory = directory
        self.master, self.slave = pty.openpty()
        self.before = termios.tcgetattr(self.slave)
        self.resize(120, 40)
        env = dict(os.environ)
        env.update(SHELL=shell_path or shutil.which("zsh"), ZDOTDIR=str(directory),
                   TERM="xterm-256color", HEART_PORTRAIT="text",
                   OLLAMA_HOST="http://127.0.0.1:9")
        for key in ["TAVILY_API_KEY", "NO_COLOR", "TERM_PROGRAM", "TMUX", "STY"]:
            env.pop(key, None)

        def session():
            os.setsid()
            fcntl.ioctl(self.slave, termios.TIOCSCTTY, 0)

        self.process = subprocess.Popen(
            [str(binary), "--dashboard", "--no-animation", "--profiles-dir", str(ROOT / "profiles")],
            stdin=self.slave, stdout=self.slave, stderr=self.slave,
            cwd=directory, env=env, preexec_fn=session,
        )
        self.output = bytearray()

    def resize(self, columns, rows):
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ,
                    struct.pack("HHHH", rows, columns, 0, 0))

    def collect(self, duration=0.1):
        until = time.monotonic() + duration
        while time.monotonic() < until:
            if select.select([self.master], [], [], min(0.05, max(0, until - time.monotonic())))[0]:
                try:
                    chunk = os.read(self.master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                self.output.extend(chunk)

    def wait(self, predicate, label, timeout=5):
        until = time.monotonic() + timeout
        while time.monotonic() < until:
            self.collect()
            if predicate():
                return
            if self.process.poll() is not None:
                raise AssertionError(f"HEART exited while waiting for {label}")
        raise AssertionError(f"Timed out waiting for {label}")

    def send(self, data):
        os.write(self.master, data if isinstance(data, bytes) else data.encode())

    def command(self, value):
        # Shell receives one line; terminal events and child output are still processed normally.
        self.send(value + "\r")

    def file(self, name):
        return self.directory / name

    def wait_file(self, name):
        self.wait(lambda: self.file(name).exists() and self.file(name).stat().st_size > 0, name)
        return self.file(name).read_text()

    def displayed(self, text):
        # Ratatui diffs reuse existing blank cells, so raw captures omit some spaces.
        data = re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]", b"", self.output)
        return re.sub(rb"\s+", b"", text.encode()) in re.sub(rb"\s+", b"", data)

    def close(self):
        for _ in range(2):
            if self.process.poll() is not None:
                break
            self.send(F12)
            self.collect(0.15)
            self.send(b"\x11")
            self.collect(0.15)
        try:
            self.process.wait(timeout=4)
        except subprocess.TimeoutExpired:
            self.process.terminate()
            self.process.wait(timeout=3)
            raise AssertionError("HEART did not exit through its own controls")
        self.collect()
        restored = termios.tcgetattr(self.slave) == self.before
        (self.directory / "terminal.bin").write_bytes(self.output)
        os.close(self.master)
        os.close(self.slave)
        return restored


def alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    # A reparented zombie cannot execute; it still needs the OS's adopter to reap it.
    status = Path(f"/proc/{pid}/stat")
    if status.exists():
        try:
            return status.read_text().rsplit(")", 1)[1].split()[0] != "Z"
        except (OSError, IndexError):
            pass
    return True


def fixture(directory):
    (directory / "completion-directory").mkdir()
    (directory / ".zshrc").write_text(
        "PROMPT='workspace> '\nRPROMPT=''\nbindkey -e\n"
        "HISTFILE=''\nSAVEHIST=0\nexport HEART_FIXTURE='configured-rc'\n"
        "alias heart_fixture='printf configured-alias'\n"
        f"print -r -- $$ > {shlex.quote(str(directory / 'shell.pid'))}\n"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/heart")
    parser.add_argument("--output-dir", type=Path)
    args = parser.parse_args()
    if not shutil.which("zsh"):
        parser.error("zsh is needed for the controlled interactive fixture")
    if not args.binary.is_file():
        parser.error("build the HEART binary first")
    directory = args.output_dir or Path(tempfile.mkdtemp(prefix="heart-workspace-check-"))
    if args.output_dir and directory.exists() and any(directory.iterdir()):
        parser.error("--output-dir must be new or empty; existing files are never overwritten")
    directory.mkdir(parents=True, exist_ok=True)
    fixture(directory)
    checks = {}
    app = Dashboard(args.binary.resolve(), directory)
    shell_pid = foreground_pid = None
    try:
        shell_pid = int(app.wait_file("shell.pid"))
        checks["interactive_configured_shell"] = True
        app.command("heart_fixture > alias.txt; printf '%s:%s' \"$HEART_FIXTURE\" \"$TERM\" > environment.txt")
        checks["rc_alias"] = app.wait_file("alias.txt") == "configured-alias"
        checks["supported_inner_term_and_environment"] = app.wait_file("environment.txt") == "configured-rc:xterm-256color"
        app.command("printf history >> history.txt")
        app.wait_file("history.txt")
        app.send(b"\x1b[A\r")
        app.wait(lambda: app.file("history.txt").read_text() == "historyhistory", "history recall")
        checks["shell_history"] = True
        app.send("printf completion > completion-d")
        app.send(b"\t")
        app.collect(0.2)
        app.send("/result.txt\r")
        checks["tab_completion"] = app.wait_file("completion-directory/result.txt") == "completion"
        app.command("printf '\\033[31mCOLOR_CHECK\\033[0m\\n'; printf color > color.txt")
        app.wait_file("color.txt")
        app.collect(0.2)
        checks["ansi_color_output"] = bool(re.search(rb"\x1b\[[0-9;]*(?:38;5;1|31|38;2;(?:128|255);0;0)(?:;|m)", app.output))
        app.command("export HEART_KEEP='shell-state'; cd completion-directory; printf state > ../state-before.txt")
        app.wait_file("state-before.txt")
        app.send(F12)
        app.collect(0.15)
        app.send(b"/uptime\r")
        app.wait(lambda: app.displayed("Your system's been running"), "assistant uptime")
        checks["heart_input_separate"] = True
        app.send(F12)
        app.collect(0.1)
        app.command("printf '%s:%s' \"${PWD##*/}\" \"$HEART_KEEP\" > ../state-after.txt; cd ..")
        checks["shell_state_survives_focus"] = app.wait_file("state-after.txt") == "completion-directory:shell-state"
        app.command("sh -c 'echo $$ > interrupt.pid; exec sleep 30'")
        interrupted = int(app.wait_file("interrupt.pid"))
        app.send(b"\x03")
        app.collect(0.15)
        app.command("printf resumed > interrupted.txt")
        checks["ctrl_c_recovers_prompt"] = app.wait_file("interrupted.txt") == "resumed" and not alive(interrupted)
        app.command("stty size > size-wide.txt")
        wide = tuple(map(int, app.wait_file("size-wide.txt").split()))
        app.resize(80, 24)
        app.collect(0.3)
        app.command("stty size > size-small.txt")
        small = tuple(map(int, app.wait_file("size-small.txt").split()))
        checks["pty_resize"] = wide != small and 0 < small[0] < 24 and 0 < small[1] <= 80
        app.command("seq 1 150; printf output > output.txt")
        app.wait_file("output.txt")
        app.send(b"\x1b[5;2~")
        app.collect(0.1)
        app.send(b"\x1b[6;2~")
        app.collect(0.1)
        checks["scrollback_keys_keep_app_alive"] = app.process.poll() is None
        app.command("sh -c 'echo $$ > foreground.pid; exec sleep 30'")
        foreground_pid = int(app.wait_file("foreground.pid"))
        app.send(F12)
        app.collect(0.15)
        app.send(b"/butler\r")
        app.wait(lambda: app.displayed("At your service"), "HEART responding during shell job")
        checks["responsive_during_foreground_job"] = True
        app.send(b"\x11")
        app.process.wait(timeout=4)
        checks["normal_exit"] = app.process.returncode == 0
        until = time.monotonic() + 2
        while time.monotonic() < until and (alive(shell_pid) or alive(foreground_pid)):
            time.sleep(0.05)
        checks["owned_process_cleanup"] = not alive(shell_pid) and not alive(foreground_pid)
    except Exception as error:
        checks["error"] = str(error)
    finally:
        try:
            checks["terminal_restored"] = app.close()
        except Exception as error:
            checks["cleanup_error"] = str(error)
        (directory / "terminal.bin").write_bytes(app.output)
        (directory / "checks.json").write_text(json.dumps(checks, indent=2) + "\n")
    if checks and all(value is True for value in checks.values()):
        for name in ["leaderless-pipeline", "pending-input", "shell-eof", "start-failure"]:
            case_dir = directory / name
            case_dir.mkdir()
            fixture(case_dir)
            case = Dashboard(args.binary.resolve(), case_dir,
                             "/nonexistent/heart-test-shell" if name == "start-failure" else None)
            case_pid = case_session = None
            case_closed = False
            try:
                if name == "leaderless-pipeline":
                    case_session = int(case.wait_file("shell.pid"))
                    case.command('true | sh -c \'trap "" HUP TERM; echo $$ > pipeline.pid; exec sleep 30\'')
                    case_pid = int(case.wait_file("pipeline.pid"))
                    group = os.getpgid(case_pid)

                    def leader_absent():
                        try:
                            os.getpgid(group)
                            return False
                        except ProcessLookupError:
                            return True

                    case.wait(leader_absent, "exited pipeline group leader")
                    checks[name + "_fixture"] = True
                    checks[name + "_terminal_restored"] = case.close()
                    case_closed = True
                    deadline = time.monotonic() + 2
                    while time.monotonic() < deadline and alive(case_pid):
                        time.sleep(0.05)
                    checks[name + "_owned_process_cleanup"] = not alive(case_pid)
                    continue
                elif name == "pending-input":
                    case.wait_file("shell.pid")
                    case.command("trap '' HUP; printf ready > trap-ready.txt")
                    case.wait_file("trap-ready.txt")
                    case.send("printf unexpected > shutdown-executed.txt")
                    case.wait(lambda: case.displayed("shutdown-executed.txt"),
                              "typed but unsubmitted shell input")
                    checks[name + "_fixture"] = not case.file("shutdown-executed.txt").exists()
                    checks[name + "_terminal_restored"] = case.close()
                    case_closed = True
                    checks[name + "_not_executed"] = not case.file("shutdown-executed.txt").exists()
                    continue
                elif name == "shell-eof":
                    exited_pid = int(case.wait_file("shell.pid"))
                    case.send(b"\x04")
                    case.wait(lambda: not alive(exited_pid), "shell exit state")
                    case.collect(0.15)
                else:
                    case.wait(lambda: case.displayed("START FAILED"), "shell start failure")
                case.send(b"\x1bOP")
                case.wait(lambda: case.displayed("HELP"), "HEART help after shell unavailable")
                checks[name + "_heart_controls"] = case.process.poll() is None
            except Exception as error:
                checks[name + "_error"] = str(error)
            finally:
                # Only kill a surviving test child after proving it belongs to our session.
                if case_pid and alive(case_pid) and os.getsid(case_pid) == case_session:
                    os.kill(case_pid, signal.SIGKILL)
                try:
                    if not case_closed:
                        checks[name + "_terminal_restored"] = case.close()
                except Exception as error:
                    checks[name + "_cleanup_error"] = str(error)
        (directory / "checks.json").write_text(json.dumps(checks, indent=2) + "\n")
    print(json.dumps({"directory": str(directory), "checks": checks}, indent=2))
    return 0 if checks and all(value is True for value in checks.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
