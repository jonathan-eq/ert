#!/usr/bin/env python3

import asyncio
import datetime
import json

import zmq
import zmq.asyncio

context = zmq.asyncio.Context()


class ZMQClient:
    def __init__(self, name: bytes):
        socket: zmq.asyncio.Socket = context.socket(zmq.DEALER)
        socket.connect("tcp://localhost:8888")
        socket.setsockopt_string(zmq.IDENTITY, name.decode("utf-8"))
        self.socket = socket
        self.name = name
        self.recv_task = asyncio.create_task(self.recv(socket))

    async def recv(self, socket: zmq.asyncio.Socket):
        while True:
            _, frame = await socket.recv_multipart()
            print(f"""{self.name} got {frame.decode("utf-8")}""")

    async def connect(self):
        print(f"{self.name} connecting")
        await self.socket.send_multipart([self.name, b"", b"CONNECT"])

    async def disconnect(self):
        print(f"{self.name} disconnecting")
        await self.socket.send_multipart([self.name, b"", b"DISCONNECT"])

    async def send(self, message: bytes):
        await self.socket.send_multipart([self.name, b"", message])


async def main():
    ert = ZMQClient(b"ert-jonak")
    await ert.connect()
    monitor = ZMQClient(b"client-jonak")
    await monitor.connect()
    dispatcher = ZMQClient(b"dispatcher-jonak")
    await dispatcher.send(
        json.dumps(
            {
                "event_type": "forward_model_step.start",
                "time": str(datetime.datetime.now(datetime.UTC)),
                "fm_step": "1",
                "real": "0",
                "std_err": "",
                "std_out": "",
            }
        ).encode("utf-8")
    )
    await asyncio.sleep(10)
    await ert.disconnect()
    await dispatcher.disconnect()
    await monitor.disconnect()
    await asyncio.sleep(10)
    ert.recv_task.cancel()
    monitor.recv_task.cancel()
    dispatcher.recv_task.cancel()


asyncio.run(main())
