
import json
import socket
from threading import Thread
from pprint import pprint

import matplotlib.pyplot as plt
import matplotlib.animation as animation

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
    c = Thread(target=client)
    c.start()

    def average_distance(i):
        ax1.clear()
        ax1.plot(values.get('t', []), values.get('d_average_distance', []), marker='.')

    fig = plt.figure()
    ax1 = fig.add_subplot(1, 1, 1)
    anim1 = animation.FuncAnimation(fig, average_distance, interval=1000)
    plt.show()

if __name__ == '__main__':
    main()