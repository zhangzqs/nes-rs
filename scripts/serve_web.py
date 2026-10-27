#!/usr/bin/env python3
"""Serve the built browser emulator on localhost; optionally expose one local ROM."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlencode, urlsplit


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--rom", type=Path, help="Offer this local ROM in the browser")
    args = parser.parse_args()
    web = Path(__file__).resolve().parents[1] / "nes-web"
    if not (web / "pkg/nes_web_bg.wasm").exists():
        parser.error("Build first: wasm-pack build nes-web --target web --release --locked")
    if args.rom and not args.rom.is_file():
        parser.error(f"ROM not found: {args.rom}")

    class Handler(SimpleHTTPRequestHandler):
        def do_GET(self):
            if urlsplit(self.path).path == "/local-game.nes" and args.rom:
                data = args.rom.read_bytes()
                self.send_response(200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
            else:
                super().do_GET()

    url = f"http://localhost:{args.port}/"
    if args.rom:
        url += "?" + urlencode({"rom": "local-game.nes", "name": args.rom.stem})
    with ThreadingHTTPServer(("127.0.0.1", args.port), partial(Handler, directory=str(web))) as server:
        print(url, flush=True)
        server.serve_forever()


if __name__ == "__main__":
    main()
