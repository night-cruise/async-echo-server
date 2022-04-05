import socket
import threading

HOST = '127.0.0.1'
PORT = 8080


def send_request():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((HOST, PORT))
    for i in range(1, 1025):
        s.send(f"HELLO WORLD[{i}]".encode())
        data = s.recv(1024).decode()
        print(f"RECEIVE DATA: '{data}' in THREAD[{threading.currentThread().name}]")
    s.close()


def main():
    t_lst = []
    for _ in range(10):
        t = threading.Thread(target=send_request)
        t_lst.append(t)
        t.start()

    for t in t_lst:
        t.join()


if __name__ == '__main__':
    main()
