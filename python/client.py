
import json
import socket
import time

# class Serializable:
#     def __init__(self):
#         pass
#     def deserialize(self):
#         raise NotImplementedError
#     def serialize(self):
#         raise NotImplementedError

class MyStruct:
    def __init__(self, big, small):
        self.big = big
        self.small = small

    @staticmethod
    def deserialize(json_str):
        dict = json.loads(json_str)
        return MyStruct(**dict)

    def serialize(self):
        # serialize with no whitespace
        return json.dumps(self.__dict__, separators=(',', ':'))
 

my_struct = MyStruct(128, 64)

def client_program():
    # Define the server's host and port
    # '127.0.0.1' means localhost; replace with the server's actual IP if on different machines
    host = '127.0.0.1' 
    port = 12345  # Must match the server's port

    # time.sleep(1)
    # Create a socket object using IPv4 (AF_INET) and TCP (SOCK_STREAM)
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as client_sock:
            # Connect to the server
            client_sock.connect((host, port))

            json_str = my_struct.serialize()
            size = len(json_str)
            print(f"Sending: {json_str!r}")

            # formatted_hex = ' '.join(f'{b:02x}' for b in json_str)
            # print(formatted_hex)

            # Send data to the server, encoding the string to bytes
            client_sock.sendall(size.to_bytes(4, byteorder='big'))  # Send size first
            client_sock.sendall(json_str.encode('utf-8'))
            
            # Shut down the sending side to signal the server that we are done sending data
            client_sock.shutdown(socket.SHUT_WR)

            chunks = []
            while True:
                # Receive data from the server (buffer size 1024 bytes)
                data = client_sock.recv(1024)
                if not data:
                    # If recv returns an empty bytes object, the server closed the connection
                    break
                chunks.append(data)
            
            received_message = b''.join(chunks).decode()
            print(f"Received: {received_message!r}")

        # The 'with' statement ensures the socket is automatically closed
        print("Connection closed")

    except ConnectionRefusedError:
        print("Error: The server is not reachable. Ensure the server script is running.")
    except Exception as e:
        print(f"An error occurred: {e}")

if __name__ == '__main__':
    client_program()
