import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "herdr/resolve.sh"


class Resolve(unittest.TestCase):
    def test_references_and_fallback(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            stub = root / "herdr"
            stub.write_text('#!/bin/sh\ncat "$RESPONSE"\n')
            stub.chmod(0o755)
            response = root / "pane.json"
            env = dict(os.environ, HOME=tmp, HERDR_BIN_PATH=str(stub),
                       RESPONSE=str(response), HERDR_PLUGIN_CONTEXT_JSON='{"focused_pane_id":"p1"}')

            def resolve(pane):
                response.write_text(json.dumps({"result": {"pane": pane}}))
                return subprocess.run(["bash", str(SCRIPT)], env=env, capture_output=True, text=True)

            for agent in ["claude", "codex", "pi", "prime-agent", "rusty"]:
                for kind in ["id", "path"]:
                    value = "/a path/session.jsonl" if kind == "path" else "session-id"
                    got = resolve({"agent_session": dict(agent=agent, kind=kind, value=value)})
                    self.assertEqual(got.returncode, 0, got.stderr)
                    self.assertEqual(got.stdout, f"{agent} {value}\n")
            sessions = root / ".prime/agent/sessions"
            sessions.mkdir(parents=True)
            for name, cwd, mtime in [("old", "/project", 100), ("new", "/project", 200), ("other", "/elsewhere", 300)]:
                path = sessions / f"{name}.jsonl"
                path.write_text(json.dumps(dict(type="session", version=3, cwd=cwd)) + "\n")
                os.utime(path, (mtime, mtime))
            got = resolve(dict(agent="prime-agent", cwd="/project"))
            self.assertEqual(got.returncode, 0, got.stderr)
            self.assertEqual(got.stdout, f"prime-agent {sessions}/new.jsonl\n")
            self.assertNotEqual(resolve(dict(agent="prime-agent", cwd="/missing")).returncode, 0)
            self.assertNotEqual(resolve(dict(agent="unsupported")).returncode, 0)
            self.assertNotEqual(resolve(dict(agent_session=dict(agent="pi", kind="path", value=""))).returncode, 0)


if __name__ == "__main__":
    unittest.main()
