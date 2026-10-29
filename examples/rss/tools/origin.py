"""A local stand-in origin for the fixture feeds: serves <dir>/<name>.xml with the cache controls a path asks for,
answers conditional requests with 304, and logs every request's validator headers to <log>.

  /cc/<policy>/<name>.xml   policy: max60 | nostore | nocache | none | etag | lm | both
    max60   Cache-Control: max-age=60, plus ETag and Last-Modified
    etag    ETag only;  lm  Last-Modified only;  both  ETag + Last-Modified, no Cache-Control
    nostore Cache-Control: no-store;  nocache Cache-Control: no-cache, ETag;  none  nothing
The ETag is a digest of the bytes, Last-Modified the file's mtime, so editing a fixture file changes both.
Usage: python3 -I origin.py <dir> <log>   (binds 127.0.0.1:0, prints "port N")
"""
import email.utils, hashlib, http.server, os, sys, json, time

ROOT, LOG = sys.argv[1], sys.argv[2]


class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def do_GET(self):
        parts = self.path.split("?")[0].strip("/").split("/")
        entry = {"t": round(time.time(), 3), "path": self.path,
                 "if-none-match": self.headers.get("If-None-Match"),
                 "if-modified-since": self.headers.get("If-Modified-Since"),
                 "user-agent": self.headers.get("User-Agent"), "accept-encoding": self.headers.get("Accept-Encoding")}
        if len(parts) != 3 or parts[0] != "cc":
            self.send_error(404)
            return
        policy, name = parts[1], parts[2]
        p = os.path.join(ROOT, os.path.basename(name))
        if not os.path.exists(p):
            entry["status"] = 404
            open(LOG, "a").write(json.dumps(entry) + "\n")
            self.send_error(404)
            return
        body = open(p, "rb").read()
        etag = '"' + hashlib.sha256(body).hexdigest()[:16] + '"'
        lm = email.utils.formatdate(int(os.path.getmtime(p)), usegmt=True)
        h = {"Content-Type": "application/rss+xml; charset=utf-8"}
        if policy in ("max60", "etag", "both", "nocache"):
            h["ETag"] = etag
        if policy in ("max60", "lm", "both"):
            h["Last-Modified"] = lm
        if policy == "max60":
            h["Cache-Control"] = "max-age=60"
        elif policy == "nostore":
            h["Cache-Control"] = "no-store"
        elif policy == "nocache":
            h["Cache-Control"] = "no-cache"
        inm, ims = self.headers.get("If-None-Match"), self.headers.get("If-Modified-Since")
        fresh = False
        if inm and "ETag" in h:
            fresh = inm == etag
        elif ims and "Last-Modified" in h:
            fresh = ims == lm
        status = 304 if fresh else 200
        entry["status"] = status
        open(LOG, "a").write(json.dumps(entry) + "\n")
        self.send_response(status)
        for k, v in h.items():
            if status == 304 and k == "Content-Type":
                continue
            self.send_header(k, v)
        if status == 304:
            self.end_headers()
            return
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)
print("port", srv.server_address[1], flush=True)
srv.serve_forever()
