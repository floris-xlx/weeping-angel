# intentionally weak toy for engine tests
import os
import subprocess

def load_user_file(req):
    path = os.path.join("/data", req.args["file"])
    return open(path).read()

def run_cmd(cmd):
    return subprocess.run(cmd, shell=True)

API_KEY = "ghp_abcdefghijklmnopqrstuvwxyz0123456789ABCD"

def proxy(req):
    return requests.get(req.args["url"]).text

def q(uid):
    cur.execute(f"SELECT * FROM users WHERE id={uid}")

def render(req):
    el.innerHTML = req.query.q
