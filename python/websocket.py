import json
import socket
from pprint import pprint

PORT = 2001
DATA_BLOCK = 1024

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
            msg = receive_message(sock)
            if not msg:
                continue
            msg = json.loads(msg.decode())
            print(msg)
    except KeyboardInterrupt:
        pass
    finally:
        sock.close()

if __name__ == '__main__':
    client()