#!/usr/bin/env python3
"""lcb.py FILE.jsonl — run control requests from a JSON-lines file ({"method":..., "params":...} or {"sleep": s})."""
import json, os, socket, sys, time

def token():
    """The app's control token (docs/control-protocol.md: Authentication)."""
    if os.environ.get("LIGHTCRAFT_CONTROL_TOKEN"):
        return os.environ["LIGHTCRAFT_CONTROL_TOKEN"].strip()
    path = os.environ.get("LIGHTCRAFT_CONTROL_TOKEN_FILE")
    if not path:
        cfg = os.environ.get("LIGHTCRAFT_CONFIG_DIR")
        if not cfg:
            home = os.path.expanduser("~")
            if sys.platform == "darwin":
                cfg = os.path.join(home, "Library/Application Support/LightCraft")
            elif os.name == "nt":
                cfg = os.path.join(os.environ.get("APPDATA", home), "LightCraft")
            else:
                cfg = os.path.join(os.environ.get("XDG_CONFIG_HOME", os.path.join(home, ".config")), "lightcraft")
        path = os.path.join(cfg, "control-token")
    with open(path) as f:
        return f.read().strip()

def line(s):
    buf = b""
    while not buf.endswith(b"\n"):
        c = s.recv(1 << 20)
        if not c: break
        buf += c
    return json.loads(buf)

def call(method, params):
    s = socket.create_connection(("127.0.0.1", 7980), timeout=180)
    s.sendall((json.dumps({"id": "auth", "method": "auth", "params": {"token": token()}}) + "\n").encode())
    auth = line(s)
    if not auth.get("ok"):
        return auth
    s.sendall((json.dumps({"id": 1, "method": method, "params": params}) + "\n").encode())
    return line(s)
for line in open(sys.argv[1]):
    line = line.strip()
    if not line or line.startswith("#"): continue
    r = json.loads(line)
    if "sleep" in r:
        time.sleep(r["sleep"]); continue
    out = call(r["method"], r.get("params", {}))
    print(r["method"], json.dumps(out)[:160])
