
import json
import socket
import sys
from threading import Thread

from statistics import statistics_figure

PORT = 2001
DATA_BLOCK = 1024

values = {}

def receive_message(sock: socket.socket) -> str:
    data = []

    length_raw = sock.recv(4)
    if len(length_raw) != 4:
        exit(0)

    length = int.from_bytes(length_raw, sys.byteorder)
    received = 0

    while received < length:
        chunk = sock.recv(length - received)

        if not chunk:
            exit(0)

        data.append(chunk)
        received += len(chunk)

    return b''.join(data)

def client():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(('localhost', PORT))

    try:
        while True:
            raw_msg = receive_message(sock)
            if raw_msg:
                msg = json.loads(raw_msg.decode())
                for k, v in msg.items():
                    values.setdefault(k, []).append(v)
    except KeyboardInterrupt:
        pass
    finally:
        sock.close()

def main():
    client_thread = Thread(target=client)
    client_thread.start()

    statistics_figure(values)

if __name__ == '__main__':
    main()
