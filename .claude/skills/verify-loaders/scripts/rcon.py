#!/usr/bin/env python3
"""Tiny RCON client: rcon.py <port> <command...>   (password bmh). Prints the server's answer."""
import socket
import struct
import sys


def packet(req_id, kind, body):
    payload = struct.pack("<ii", req_id, kind) + body.encode("utf-8") + b"\x00\x00"
    return struct.pack("<i", len(payload)) + payload


def read_packet(sock):
    raw = b""
    while len(raw) < 4:
        chunk = sock.recv(4 - len(raw))
        if not chunk:
            raise ConnectionError("closed")
        raw += chunk
    (length,) = struct.unpack("<i", raw)
    data = b""
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk:
            raise ConnectionError("closed")
        data += chunk
    req_id, kind = struct.unpack("<ii", data[:8])
    return req_id, kind, data[8:-2].decode("utf-8", "replace")


def main():
    port = int(sys.argv[1])
    command = " ".join(sys.argv[2:])
    sock = socket.create_connection(("127.0.0.1", port), timeout=15)
    sock.sendall(packet(1, 3, "bmh"))
    req_id, _, _ = read_packet(sock)
    if req_id == -1:
        print("RCON: bad password")
        sys.exit(2)
    sock.sendall(packet(2, 2, command))
    try:
        _, _, body = read_packet(sock)
        print(body)
    except (ConnectionError, socket.timeout):
        # `stop` closes the connection before answering
        print("(connection closed)")
    sock.close()


main()
