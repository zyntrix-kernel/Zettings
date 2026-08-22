from __future__ import annotations

import os
import re
import sys
import time
import threading
from pathlib import Path

import pyte
from winpty import PtyProcess


# ============================================================
# CONFIGURATION
# ============================================================

# The project directory where OpenCode will start.
#
# Because this Python file is inside your Settings-app project,
# this automatically points to that project.
PROJECT_DIR = Path(__file__).resolve().parent


# Command used to launch OpenCode.
OPENCODE_COMMAND = "opencode"


# Additional OpenCode arguments, if you ever need them.
#
# Example:
# OPENCODE_ARGS = ["--model", "some-model"]
#
OPENCODE_ARGS: list[str] = []


# How long OpenCode must appear stopped before we send
# "continue".
#
# 5 seconds is deliberately conservative.
STOP_CONFIRM_SECONDS = 5.0


# Minimum time between automatic "continue" messages.
#
# This prevents accidental rapid looping.
CONTINUE_COOLDOWN_SECONDS = 15.0


# How often the detector checks the virtual terminal screen.
CHECK_INTERVAL_SECONDS = 0.25


# Text to send.
CONTINUE_TEXT = "continue"


# ============================================================
# DETECTION SETTINGS
# ============================================================

# This is the important indicator from your screenshot.
#
# While OpenCode is generating:
#
#     esc interrupt
#
# When generation stops, it disappears.
ESC_INTERRUPT_RE = re.compile(
    r"\besc\s+interrupt\b",
    re.IGNORECASE,
)


# Your OpenCode screen also shows this when the interactive
# command/input area is available.
CTRL_P_COMMANDS_RE = re.compile(
    r"ctrl\+p\s+commands",
    re.IGNORECASE,
)


# These are useful secondary indicators.
COMMAND_RE = re.compile(
    r"/(?:agents|compact|connect|debug|diff|editor|exit|export|fork)\b",
    re.IGNORECASE,
)


# Don't automatically continue if the visible screen appears
# to be asking for a manual decision.
#
# These are intentionally conservative.
MANUAL_CONFIRM_RE = re.compile(
    r"\b(?:permission|approval|required|approve|allow|confirm)\b",
    re.IGNORECASE,
)


# ============================================================
# TERMINAL SIZE
# ============================================================

def get_terminal_size() -> tuple[int, int]:
    """
    Get the size of the terminal that is running this Python
    wrapper.

    Returns:
        (rows, cols)
    """

    try:
        size = os.get_terminal_size(sys.stdout.fileno())

        cols = max(40, size.columns)
        rows = max(12, size.lines)

        return rows, cols

    except Exception:
        # Safe fallback.
        return 36, 120


# ============================================================
# VIRTUAL TERMINAL
# ============================================================

class VirtualTerminal:
    """
    Maintains an in-memory representation of the terminal.

    OpenCode doesn't simply print lines one after another.
    It moves the cursor around and redraws the screen using
    ANSI/VT escape sequences.

    pyte interprets those sequences for us.
    """

    def __init__(self, rows: int, cols: int):

        self.rows = rows
        self.cols = cols

        self.screen = pyte.Screen(
            cols,
            rows,
        )

        self.stream = pyte.Stream(
            self.screen,
        )

        self.lock = threading.Lock()

    def feed(self, text: str) -> None:

        if not text:
            return

        with self.lock:

            try:
                self.stream.feed(text)

            except Exception as exc:

                # Never allow a malformed terminal sequence to
                # kill the OpenCode process.
                print(
                    f"\n[PTY PARSER WARNING] {exc}\n",
                    file=sys.stderr,
                )

    def get_screen(self) -> str:

        with self.lock:

            # pyte.screen.display represents the visible terminal
            # contents rather than raw ANSI output.
            lines = self.screen.display

            return "\n".join(lines)


# ============================================================
# OPENCODE DETECTOR
# ============================================================

class OpenCodeDetector:

    def __init__(self, terminal: VirtualTerminal):

        self.terminal = terminal

        self.lock = threading.Lock()

        # Have we ever seen OpenCode actively generating?
        self.active_seen = False

        # Time at which the potential idle state began.
        self.idle_started_at: float | None = None

        # Last time we sent continue.
        self.last_continue_at = 0.0

        # Prevent multiple continuations from being sent at
        # exactly the same time.
        self.continuation_in_progress = False

        # Last detector state.
        self.last_state = "starting"

    # --------------------------------------------------------
    # SCREEN HELPERS
    # --------------------------------------------------------

    def screen_text(self) -> str:

        return self.terminal.get_screen()

    def has_interrupt_indicator(self, text: str) -> bool:

        return bool(
            ESC_INTERRUPT_RE.search(text)
        )

    def has_command_area(self, text: str) -> bool:

        return bool(
            CTRL_P_COMMANDS_RE.search(text)
        )

    def has_commands(self, text: str) -> bool:

        return bool(
            COMMAND_RE.search(text)
        )

    def has_manual_confirmation(self, text: str) -> bool:

        return bool(
            MANUAL_CONFIRM_RE.search(text)
        )

    # --------------------------------------------------------
    # STATE DETECTION
    # --------------------------------------------------------

    def detect(self) -> str:

        text = self.screen_text()

        if not text.strip():
            return "starting"

        has_interrupt = self.has_interrupt_indicator(text)

        has_command_area = self.has_command_area(text)

        has_commands = self.has_commands(text)

        has_manual = self.has_manual_confirmation(text)

        # ----------------------------------------------------
        # MANUAL DECISION
        # ----------------------------------------------------

        if has_manual:
            return "manual"

        # ----------------------------------------------------
        # ACTIVE
        # ----------------------------------------------------

        # This is the strongest signal from your screenshot.
        #
        # While OpenCode is working:
        #
        #     esc interrupt
        #
        # is visible.
        if has_interrupt:

            return "active"

        # ----------------------------------------------------
        # IDLE
        # ----------------------------------------------------

        # Once "esc interrupt" disappears, require additional
        # evidence that the normal interactive TUI is back.
        #
        # Your screenshot shows:
        #
        #     ctrl+p commands
        #
        # and command entries.
        if has_command_area and has_commands:

            return "idle"

        # ----------------------------------------------------
        # UNKNOWN
        # ----------------------------------------------------

        return "unknown"

    # --------------------------------------------------------
    # STATE UPDATE
    # --------------------------------------------------------

    def update(self) -> tuple[str, float]:

        state = self.detect()

        now = time.monotonic()

        with self.lock:

            # ------------------------------------------------
            # ACTIVE
            # ------------------------------------------------

            if state == "active":

                self.active_seen = True

                self.idle_started_at = None

                self.continuation_in_progress = False

            # ------------------------------------------------
            # IDLE
            # ------------------------------------------------

            elif state == "idle":

                # We NEVER automatically continue immediately
                # when starting the program.
                #
                # OpenCode must first have been observed actively
                # generating.
                if self.active_seen:

                    if self.idle_started_at is None:

                        self.idle_started_at = now

                    idle_for = (
                        now - self.idle_started_at
                    )

                    return state, idle_for

            # ------------------------------------------------
            # EVERYTHING ELSE
            # ------------------------------------------------

            else:

                self.idle_started_at = None

        return state, 0.0

    # --------------------------------------------------------
    # SHOULD CONTINUE?
    # --------------------------------------------------------

    def should_continue(self) -> bool:

        now = time.monotonic()

        with self.lock:

            if not self.active_seen:
                return False

            if self.continuation_in_progress:
                return False

            if self.idle_started_at is None:
                return False

            idle_for = (
                now - self.idle_started_at
            )

            if idle_for < STOP_CONFIRM_SECONDS:
                return False

            if (
                now - self.last_continue_at
                < CONTINUE_COOLDOWN_SECONDS
            ):
                return False

            self.continuation_in_progress = True

            return True

    # --------------------------------------------------------
    # MARK CONTINUE SENT
    # --------------------------------------------------------

    def mark_continue_sent(self):

        with self.lock:

            self.last_continue_at = time.monotonic()

            self.active_seen = False

            self.idle_started_at = None

            self.continuation_in_progress = False


# ============================================================
# PTY OUTPUT READER
# ============================================================

class PTYReader:

    def __init__(
        self,
        pty: PtyProcess,
        terminal: VirtualTerminal,
    ):

        self.pty = pty
        self.terminal = terminal

        self.running = True

        self.thread = threading.Thread(
            target=self._run,
            daemon=True,
        )

    def start(self):

        self.thread.start()

    def _run(self):

        while self.running:

            try:

                data = self.pty.read(4096)

            except EOFError:

                break

            except Exception as exc:

                print(
                    f"\n[PTY READ ERROR] {exc}\n",
                    file=sys.stderr,
                )

                break

            if not data:

                time.sleep(0.01)

                continue

            if isinstance(data, bytes):

                text = data.decode(
                    "utf-8",
                    errors="replace",
                )

            else:

                text = str(data)

            # Feed the terminal emulator.
            self.terminal.feed(text)

            # Pass the original terminal stream through to the
            # actual Windows console.
            #
            # This is what lets you continue seeing OpenCode
            # normally.
            try:

                sys.stdout.write(text)
                sys.stdout.flush()

            except Exception:

                pass

        self.running = False

    def stop(self):

        self.running = False


# ============================================================
# SEND CONTINUE
# ============================================================

def send_continue(
    pty: PtyProcess,
    detector: OpenCodeDetector,
):

    print()
    print()
    print(
        "============================================================"
    )
    print(
        " AUTO-CONTINUE: OpenCode appears to have stopped"
    )
    print(
        " Sending: continue"
    )
    print(
        "============================================================"
    )
    print()

    try:

        # Send the literal text.
        pty.write(
            CONTINUE_TEXT
        )

        # Give the TUI a moment to receive it.
        time.sleep(0.15)

        # CR is the terminal Enter key.
        pty.write(
            "\r"
        )

        detector.mark_continue_sent()

        return True

    except Exception as exc:

        print()
        print(
            f"[AUTO-CONTINUE ERROR] {exc}"
        )
        print()

        with detector.lock:

            detector.continuation_in_progress = False

        return False


# ============================================================
# DETECTOR LOOP
# ============================================================

def detector_loop(
    pty: PtyProcess,
    detector: OpenCodeDetector,
):

    previous_state = None

    while pty.isalive():

        state, idle_for = detector.update()

        # Only print state changes, otherwise diagnostic text
        # would constantly interfere with the TUI.
        if state != previous_state:

            if state == "active":

                print(
                    "\n[AUTO] OpenCode is ACTIVE."
                )

            elif state == "idle":

                print(
                    "\n[AUTO] OpenCode appears IDLE. "
                    "Confirming..."
                )

            elif state == "manual":

                print(
                    "\n[AUTO] Manual decision detected. "
                    "Automation paused."
                )

            elif state == "unknown":

                print(
                    "\n[AUTO] Terminal state unclear. "
                    "Waiting."
                )

            previous_state = state

        # ----------------------------------------------------
        # CONFIRM STOP
        # ----------------------------------------------------

        if state == "idle":

            if detector.should_continue():

                send_continue(
                    pty,
                    detector,
                )

        time.sleep(
            CHECK_INTERVAL_SECONDS
        )


# ============================================================
# START OPENCODE
# ============================================================

def start_opencode():

    rows, cols = get_terminal_size()

    print()
    print(
        "============================================================"
    )
    print(
        " OpenCode Auto-Continue"
    )
    print(
        "============================================================"
    )
    print()
    print(
        f"Project : {PROJECT_DIR}"
    )
    print(
        f"PTY     : {rows} rows × {cols} columns"
    )
    print(
        f"Confirm : {STOP_CONFIRM_SECONDS} seconds"
    )
    print(
        f"Cooldown: {CONTINUE_COOLDOWN_SECONDS} seconds"
    )
    print()
    print(
        "Starting OpenCode..."
    )
    print()

    # --------------------------------------------------------
    # Environment
    # --------------------------------------------------------

    env = os.environ.copy()

    # Tell OpenCode and other terminal applications that this
    # is an xterm-compatible terminal.
    env["TERM"] = "xterm-256color"

    env["COLORTERM"] = "truecolor"

    # --------------------------------------------------------
    # Command
    # --------------------------------------------------------

    command = [
        OPENCODE_COMMAND,
        *OPENCODE_ARGS,
    ]

    # --------------------------------------------------------
    # PTY
    # --------------------------------------------------------

    #
    # IMPORTANT:
    #
    # pywinpty expects:
    #
    #     dimensions=(rows, cols)
    #
    # Internally it creates the Windows PTY using cols/rows.
    #
    pty = PtyProcess.spawn(
        command,
        cwd=str(PROJECT_DIR),
        env=env,
        dimensions=(
            rows,
            cols,
        ),
    )

    return pty, rows, cols


# ============================================================
# MAIN
# ============================================================

def main():

    pty = None
    reader = None

    try:

        pty, rows, cols = start_opencode()

        # Create the virtual terminal.
        terminal = VirtualTerminal(
            rows,
            cols,
        )

        # Detector operates on the virtual screen.
        detector = OpenCodeDetector(
            terminal
        )

        # Read OpenCode's PTY output.
        reader = PTYReader(
            pty,
            terminal,
        )

        reader.start()

        print()
        print(
            "[AUTO] Monitoring OpenCode..."
        )
        print(
            "[AUTO] Ctrl+C = stop wrapper + OpenCode"
        )
        print()

        # Start detection.
        detector_loop(
            pty,
            detector,
        )

    except KeyboardInterrupt:

        print()
        print()
        print(
            "Stopping Auto-Continue..."
        )

    except FileNotFoundError:

        print()
        print(
            "ERROR: OpenCode executable was not found."
        )
        print()
        print(
            "Test this in PowerShell:"
        )
        print()
        print(
            "    opencode --version"
        )
        print()

        sys.exit(1)

    except Exception as exc:

        print()
        print(
            "============================================================"
        )
        print(
            " ERROR"
        )
        print(
            "============================================================"
        )
        print()
        print(
            repr(exc)
        )
        print()

        raise

    finally:

        if reader is not None:

            reader.stop()

        if pty is not None:

            try:

                if pty.isalive():

                    pty.terminate()

            except Exception:

                pass

        print()
        print(
            "Auto-Continue exited."
        )


# ============================================================
# ENTRY POINT
# ============================================================

if __name__ == "__main__":
    main()