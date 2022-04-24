from pwn import *
import time

ADDRESS = "127.0.0.1"
PORT = 8080
KEY = b"\x03\xa4\x4f\x11\xdd\xb7\xfd\x2b\x66\x16\x5a\xd4\x5d\xec\xcd"

def main():
    s = remote(ADDRESS, PORT)
    for k in KEY:
        s.send(bytes([k]))
        print(s.recv(1))
    
    time.sleep(1)
    print(s.recv())

if __name__ == "__main__":
    main()
