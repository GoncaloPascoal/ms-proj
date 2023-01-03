
import json
import socket
from threading import Thread

from sim_statistics import statistics_figure

PORT = 2001
DATA_BLOCK = 1024

values = {}

def receive_message(sock: socket.socket) -> str:
    data = []
    while True:
        try:
            chunk = sock.recv(DATA_BLOCK)
        except BlockingIOError:
            continue
        if not chunk or chunk.find(b'\x00\x00') != -1:
            break
        data.append(chunk)
    return b''.join(data)

def client():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(('localhost', PORT))
    sock.setblocking(False)

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
