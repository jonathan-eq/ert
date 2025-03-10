#!/usr/bin/env python3

import datetime
import json
import time

import zmq
context = zmq.Context()
socket = context.socket(zmq.DEALER)
socket.connect("tcp://localhost:8889")
socket.setsockopt(zmq.IDENTITY, b"identity_jonak")

def client():
  
  socket.send_multipart([b"client-jonak", b"",b"CONNECT"])
  rec_msgs = socket.recv_multipart()

  for rec_msg in rec_msgs:
    print(rec_msg.decode("utf-8"))

  conf_msg = socket.recv_multipart()
  for rec_msg in conf_msg:
    print(rec_msg.decode("utf-8"))
  socket.send_multipart([b"client-jonak", b"",b"DISCONNECT"])

def dispatcher():
  socket.send_multipart([b"dispatcher-jonak", b"",b"CONNECT"])
  rec_msgs = socket.recv_multipart()

  for rec_msg in rec_msgs:
    print(rec_msg.decode("utf-8"))

  print(f"\n\n")
  socket.send_multipart([b"dispatcher-jonak", b"", json.dumps({"event_type": "forward_model.start","time": str(datetime.datetime.now(datetime.timezone.utc)), "fm_step": "1", "real_id": "0", "stderr": "", "stdout": ""}).encode("utf-8")])
  conf_msg = socket.recv_multipart()
  for rec_msg in conf_msg:
    print(rec_msg.decode("utf-8"))
  socket.send_multipart([b"dispatcher-jonak", b"",b"DISCONNECT"])
  #time.sleep(3)
  #socket.send_multipart([b"DISCONNECT",b"", "client-jonak"])
  
dispatcher()
#time.sleep(3)
#client()