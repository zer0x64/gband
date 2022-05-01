import os

def main():
    dirname = os.path.dirname(__file__)
    filename=os.path.join(dirname, "../res/map_tilemap.bin")
    logic_filename=os.path.join(dirname, "../res/map_logic.bin")
    data = []
    try:
        with open(filename, "rb") as f:
            byte = f.read(1)

            while byte:
                data.append(operation(byte=byte))
                byte = f.read(1)

            with open(logic_filename, "wb") as l:
                for x in data:
                    l.write(x)

            print("success!")
    except IOError:
        print('Error While Opening the file!')

def operation(byte):
    val = b'\x00'

    #felt lazy with that condition
    if byte != b'\x8E' and byte != b'\x8F' and byte != b'\x9E' and byte != b'\x9F':
        val = b'\x01'

    return val

main()