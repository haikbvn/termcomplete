"""Exercise the actual PTY binary without editing user shell startup files."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import shlex
import signal
import struct
import subprocess
import tempfile
import termios
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--bin", type=Path, required=True)
args = parser.parse_args()
binary = args.bin.resolve()

def wait_for(fd, expected, transcript, timeout=12):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.1)
        if ready:
            try:
                chunk = os.read(fd, 65536)
            except OSError:
                break
            if not chunk:
                break
            transcript.extend(chunk)
            if expected in transcript:
                return
    raise AssertionError(f"PTY did not produce {expected!r}; tail={bytes(transcript[-1500:])!r}")

results = []
for shell, options in [("/bin/bash", ["--noprofile", "--norc", "-i"]), ("/bin/zsh", ["-f", "-i"])]:
    with tempfile.TemporaryDirectory(prefix="tc-pty-", dir="/private/tmp") as temporary:
        root = Path(temporary)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
        env = dict(os.environ, TERM="xterm-256color", INPUTRC="/dev/null", LANG="en_US.UTF-8", LC_ALL="en_US.UTF-8", PS1="TCOMP_PROMPT> ",
                   TCOMP_DATA_DIR=str(root / "data"), BASH_SILENCE_DEPRECATION_WARNING="1")
        # Exercise the actual default macOS socket path, including its length.
        env.pop("TCOMP_RUNTIME_DIR", None)
        def session():
            os.setsid()
            fcntl.ioctl(0, termios.TIOCSCTTY, 0)
        process = subprocess.Popen([str(binary), "--", shell, *options], stdin=slave, stdout=slave, stderr=slave,
                                   env=env, start_new_session=False, preexec_fn=session)
        os.close(slave)
        transcript = bytearray()
        try:
            wait_for(master, b"TCOMP_PROMPT> ", transcript)
            assert b"Termcomplete had an Error" not in transcript
            os.write(master, b"printf '\\nTCOMP_READY\\n'\r")
            wait_for(master, b"\r\nTCOMP_READY\r\n", transcript)
            text = "quoted path / tiếng Việt / 'single' / \"double\""
            os.write(master, ("printf '%s\\n' " + shlex.quote(text) + "\r").encode())
            wait_for(master, b"\r\n" + text.encode() + b"\r\n", transcript)
            os.write(master, b"printf '%s\\n' CURSOR_AB\x1b[DX\r")
            wait_for(master, b"\r\nCURSOR_AXB\r\n", transcript)
            fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack("HHHH", 37, 120, 0, 0))
            # A resize can redraw the prompt between command echo and output.
            # Use a distinct output marker rather than depending on that echo.
            os.write(master, b"printf '\\nTCOMP_SIZE=%s\\n' \"$(stty size)\"\r")
            wait_for(master, b"\r\nTCOMP_SIZE=37 120\r\n", transcript)
            os.write(master, b"sleep 30\r")
            time.sleep(0.2)
            os.write(master, b"\x03printf '\\nTCOMP_CANCELLED\\n'\r")
            wait_for(master, b"\r\nTCOMP_CANCELLED\r\n", transcript)
            os.write(master, b"exit\r")
            deadline = time.monotonic() + 6
            while process.poll() is None and time.monotonic() < deadline:
                ready, _, _ = select.select([master], [], [], 0.1)
                if ready:
                    try: transcript.extend(os.read(master, 65536))
                    except OSError: break
            assert process.poll() is not None, f"Shell did not exit: {bytes(transcript[-1500:])!r}"
            assert process.returncode == 0, process.returncode
            results.append({"shell": shell, "passed": ["startup", "quoted Unicode", "cursor movement", "resize", "interrupt", "clean exit"]})
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=3)
            os.close(master)
print(json.dumps({"binary": str(binary), "results": results}, indent=2))
