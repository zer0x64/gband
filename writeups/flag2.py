from pwn import *
import time

ADDRESS = "9000:cafe:1234:5678:216:3eff:fef4:94cb"
PORT = 8080
KEY = b"\x03\xa4\x4f\x11\xdd\xb7\xfd\x2b\x66\x16\x5a\xd4\x5d\xec\xcd"

def main():
    s = remote(ADDRESS, PORT)
    for k in KEY:
        s.send(bytes([k]))
        print(s.recv(1))
    
    for i in range(0, 32):
        print(s.recv(16))

if __name__ == "__main__":
    main()
