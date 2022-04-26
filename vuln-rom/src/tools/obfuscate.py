import binascii
from sys import argv

key = binascii.unhexlify("7c6b874523db659911aef3a742b94802")

with open(argv[1], "rb") as input:
    with open(argv[2], "wb") as output:
        output.write(bytes((byte ^ key[i % 16]  for (i, byte) in enumerate(input.read()))))