#!/usr/bin/env python3

import time

import zmq


context = zmq.Context()
socket = context.socket(zmq.DEALER)
socket.connect("tcp://localhost:8889")
socket.setsockopt(zmq.IDENTITY, b"client-jonak")
socket.send_multipart([b"client-jonak", b"",b"CONNECT"])
rec_msgs = socket.recv_multipart()

for rec_msg in rec_msgs:
  print(rec_msg.decode("utf-8"))

print(f"\n\n\n\n\n")
rec_msgs = socket.recv_multipart()

for rec_msg in rec_msgs:
  print(rec_msg.decode("utf-8"))
#time.sleep(3)
#socket.send_multipart([b"DISCONNECT",b"", "client-jonak"])
