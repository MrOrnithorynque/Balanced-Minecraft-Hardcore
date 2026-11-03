#!/usr/bin/env python3
"""Tiny RCON client: rcon.py <port> <password> <command...>  (prints the reply)."""
import socket, struct, sys

def packet(pid, ptype, body):
    payload = struct.pack("<ii", pid, ptype) + body.encode() + b"\x00\x00"
    return struct.pack("<i", len(payload)) + payload

def read(sock):
    head = sock.recv(4)
    if len(head) < 4:
        raise EOFError
    (length,) = struct.unpack("<i", head)
    data = b""
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk:
            raise EOFError
        data += chunk
    pid, ptype = struct.unpack("<ii", data[:8])
    return pid, ptype, data[8:-2].decode(errors="replace")

port, password = int(sys.argv[1]), sys.argv[2]
command = " ".join(sys.argv[3:])
s = socket.create_connection(("127.0.0.1", port), timeout=10)
s.sendall(packet(1, 3, password))
pid, _, _ = read(s)
if pid == -1:
    print("RCON auth failed")
    sys.exit(2)
s.sendall(packet(2, 2, command))
try:
    print(read(s)[2])
except (EOFError, socket.timeout):
    print("(no reply)")
