#!/usr/bin/env python3
"""Local ASR stand-in for Sayline end-to-end tests.

Implements the ZCode ticket endpoint and the realtime WebSocket protocol.
No audio leaves this process; it only counts bytes and returns a fixed text.
"""
import argparse
import asyncio
import json
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import websockets

STATE = {
    "tickets": set(),
    "sessions": [],
    "ticket_requests": [],
}


class TicketHandler(BaseHTTPRequestHandler):
    server_version = "SaylineAsrMock/0.1"

    def log_message(self, fmt, *args):
        print(f"[http] {fmt % args}", flush=True)

    def _json(self, status, payload):
        body = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(length).decode("utf-8", "replace") if length else ""
        if self.path == "/api/v1/asr/tickets":
            STATE["ticket_requests"].append(
                {"authorization": self.headers.get("Authorization"), "body": raw}
            )
            ticket = uuid.uuid4().hex
            STATE["tickets"].add(ticket)
            self._json(
                200,
                {
                    "code": 0,
                    "msg": "",
                    "data": {
                        "ticket": ticket,
                        "expires_in": 60,
                        "ws_path": f"ws://127.0.0.1:{WS_PORT}/api/v1/asr/realtime",
                    },
                },
            )
            return
        self._json(404, {"code": 404, "msg": "not found"})

    def do_GET(self):
        if self.path == "/__state":
            self._json(
                200,
                {
                    "ticket_requests": STATE["ticket_requests"],
                    "sessions": STATE["sessions"],
                },
            )
            return
        self._json(404, {"code": 404, "msg": "not found"})


async def asr_handler(websocket, path):
    ticket = ""
    if "?" in path:
        query = path.split("?", 1)[1]
        for pair in query.split("&"):
            key, _, value = pair.partition("=")
            if key == "ticket":
                ticket = value
    if ticket not in STATE["tickets"]:
        await websocket.close(code=1008, reason="invalid ticket")
        return
    STATE["tickets"].discard(ticket)

    session = {
        "start": None,
        "audio_bytes": 0,
        "controls": [],
        "closed": False,
    }
    STATE["sessions"].append(session)
    try:
        async for message in websocket:
            if isinstance(message, bytes):
                session["audio_bytes"] += len(message)
                continue
            payload = json.loads(message)
            kind = payload.get("type")
            if kind == "start":
                session["start"] = payload
                await websocket.send(
                    json.dumps(
                        {
                            "type": "started",
                            "provider": "sayline-mock",
                            "max_audio_duration_ms": 1_800_000,
                            "session_id": f"mock-{len(STATE['sessions'])}",
                        }
                    )
                )
                continue
            session["controls"].append(kind)
            if kind == "finish":
                await websocket.send(
                    json.dumps({"type": "result", "text": "你好，这是 Sayline 的"})
                )
                await asyncio.sleep(0.08)
                final = "你好，这是 Sayline 的测试。"
                await websocket.send(
                    json.dumps(
                        {
                            "type": "result",
                            "text": final,
                        }
                    )
                )
                await asyncio.sleep(0.08)
                await websocket.send(
                    json.dumps(
                        {
                            "type": "completed",
                            "text": final,
                            "audio_duration_ms": session["audio_bytes"] / 32,
                        }
                    )
                )
                await websocket.close(code=1000)
                return
            if kind == "cancel":
                await websocket.send(json.dumps({"type": "cancelled"}))
                await websocket.close(code=1000)
                return
            await websocket.send(
                json.dumps({"type": "error", "code": "unsupported", "retryable": False})
            )
            return
    except websockets.ConnectionClosed:
        pass
    finally:
        session["closed"] = True
        print(
            f"[ws] session done bytes={session['audio_bytes']} controls={session['controls']}",
            flush=True,
        )


async def ws_main():
    async with websockets.serve(asr_handler, "127.0.0.1", WS_PORT):
        print(f"[ws] listening on ws://127.0.0.1:{WS_PORT}", flush=True)
        await asyncio.Future()


def main():
    global WS_PORT
    parser = argparse.ArgumentParser()
    parser.add_argument("--http-port", type=int, default=18080)
    parser.add_argument("--ws-port", type=int, default=18081)
    args = parser.parse_args()
    WS_PORT = args.ws_port
    httpd = ThreadingHTTPServer(("127.0.0.1", args.http_port), TicketHandler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    print(f"[http] listening on http://127.0.0.1:{args.http_port}", flush=True)
    try:
        asyncio.run(ws_main())
    except KeyboardInterrupt:
        pass
    finally:
        httpd.shutdown()


if __name__ == "__main__":
    main()
