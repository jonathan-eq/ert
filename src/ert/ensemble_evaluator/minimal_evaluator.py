from __future__ import annotations

import asyncio
import datetime
import logging
import traceback
from collections.abc import Awaitable, Callable, Iterable, Sequence
from enum import Enum
from typing import Any, get_args
import uuid

from websocket import send
import zmq.asyncio

from _ert.events import (
    EEDone,
    EESnapshot,
    EESnapshotUpdate,
    EETerminated,
    EEUserCancel,
    EEUserDone,
    EnsembleCancelled,
    EnsembleFailed,
    EnsembleStarted,
    EnsembleSucceeded,
    Event,
    FMEvent,
    ForwardModelStepChecksum,
    RealizationEvent,
    dispatch_event_from_json,
    event_from_json,
    event_from_json_from_evaluator,
    event_to_json,
)
from _ert.forward_model_runner.client import (
    ACK_MSG,
    CONNECT_MSG,
    DISCONNECT_MSG,
    HEARTBEAT_MSG,
    HEARTBEAT_TIMEOUT,
    Client,
)
from ert.ensemble_evaluator import identifiers as ids
from ert.ensemble_evaluator.monitor import EventSentinel

from ._ensemble import FMStepSnapshot
from ._ensemble import LegacyEnsemble as Ensemble
from .config import EvaluatorServerConfig
from .snapshot import EnsembleSnapshot
from .state import (
    ENSEMBLE_STATE_CANCELLED,
    ENSEMBLE_STATE_FAILED,
    ENSEMBLE_STATE_STOPPED,
)

logger = logging.getLogger(__name__)

EVENT_HANDLER = Callable[[list[Event]], Awaitable[None]]


class HeartbeatEvent(Enum):
    event = HEARTBEAT_MSG


class EvaluatorClient(Client):
    def __init__(self, ensemble: Ensemble, config: EvaluatorServerConfig):
        self._config: EvaluatorServerConfig = config
        self._ensemble: Ensemble = ensemble

        self._evaluator_to_scheduler_queue: asyncio.Queue[Any] = asyncio.Queue()
        self._scheduler_to_evaluator_queue: asyncio.Queue[Event] = asyncio.Queue()
        self._ee_tasks: list[asyncio.Task[None]] = []
        self._server_done: asyncio.Event = asyncio.Event()
        self._server_started: asyncio.Future[None] = asyncio.Future()
        self._id = str(uuid.uuid1()).split("-", maxsplit=1)[0]
        self._receiver_timeout: float = 60.0
        self._event_queue: asyncio.Queue[Event | EventSentinel] = asyncio.Queue()
        self._received_evaluator_done = asyncio.Event()
        self._ensemble_task = asyncio.create_task(
                self._ensemble.evaluate(
                    self._config, self._scheduler_to_evaluator_queue, self._evaluator_to_scheduler_queue
                ),
                name="ensemble_task",
            )
        self._forwarding_task = asyncio.create_task(self._forward_events())
        super().__init__(config.url, config.token, dealer_name=f"ert-{self._id}")
        print("EvaluatorClient __init__ merging")
        
    async def _force_refresh(self) -> None:
        await self.send(event_to_json(EESnapshotUpdate(ensemble=self._ensemble.id_, snapshot=self._ensemble.snapshot.to_dict())))
        #self._ensemble.snapshot.merge_snapshot(EnsembleSnapshot.from_nested_dict({}))
        #await self._event_queue.put(self._ensemble.snapshot)
        

    async def _forward_events(self) -> None:
      
      while True:
        event = await self._scheduler_to_evaluator_queue.get()
        await self.send(event_to_json(event))

    async def _unused_stopped_handler(self, events: Sequence[EnsembleSucceeded]) -> None:
        if self.ensemble.status == ENSEMBLE_STATE_FAILED:
            return

        max_memory_usage = -1
        for (real_id, _), fm_step in self.ensemble.snapshot.get_all_fm_steps().items():
            # Infer max memory usage
            memory_usage = fm_step.get(ids.MAX_MEMORY_USAGE) or "-1"
            max_memory_usage = max(int(memory_usage), max_memory_usage)

            if cpu_message := detect_overspent_cpu(
                self.ensemble.reals[int(real_id)].num_cpu, real_id, fm_step
            ):
                logger.warning(cpu_message)

        logger.info(
            f"Ensemble ran with maximum memory usage for a single realization job: {max_memory_usage}"
        )

        #await self._append_message(self.ensemble.update_snapshot(events))


    async def _unused_failed_handler(self, events: Sequence[EnsembleFailed]) -> None:
        if self.ensemble.status in {
            ENSEMBLE_STATE_STOPPED,
            ENSEMBLE_STATE_CANCELLED,
        }:
            return
        # if list is empty this call is not triggered by an
        # event, but as a consequence of some bad state
        # create a fake event because that's currently the only
        # api for setting state in the ensemble
        if len(events) == 0:
            events = [EnsembleFailed(ensemble=self.ensemble.id_)]
        #await self._append_message(self.ensemble.update_snapshot(events))
       # await self._signal_cancel()  # let ensemble know it should stop

    @property
    def ensemble(self) -> Ensemble:
        return self._ensemble

    async def process_message(self, msg: str):
        event = event_from_json_from_evaluator(msg)
        match event:
            case EEDone():
                #print("Minimal evaluator got EEDone")
                self._received_evaluator_done.set()
            case ForwardModelStepChecksum():
                #print("Minimal evaluator got fm checksum")
                await self._evaluator_to_scheduler_queue.put(event)
            case EESnapshotUpdate()|EESnapshot():
                #print("Minimal evaluator got EESnapshotUpdate")
                print(f"Minimal evaluator merging due to new snapshot {event.snapshot}")
                self._ensemble.snapshot.merge_snapshot(EnsembleSnapshot.from_nested_dict(event.snapshot))


      # JONAK REMEMBER TO IMPLEMENT THS
      # self._router_socket: zmq.asyncio.Socket = zmq_context.socket(zmq.ROUTER)
      #self._router_socket.setsockopt(zmq.LINGER, 0)
      #if self._config.server_public_key and self._config.server_secret_key:
      #    self._router_socket.curve_secretkey = self._config.server_secret_key
      #    self._router_socket.curve_publickey = self._config.server_public_key
      #    self._router_socket.curve_server = True  


      #event = EETerminated(ensemble=self._ensemble.id_)
      # await self._events_to_send.put(event)
      #
      #


    async def run_and_get_successful_realizations(self) -> list[int]:
        await self._received_evaluator_done.wait()
        print("Minimal evaluator is done. BYE!")
        logger.debug("Evaluator is done")
        return self._ensemble.get_successful_realizations()

    @staticmethod
    def _get_ens_id(source: str) -> str:
        # the ens_id will be found at /ert/ensemble/ens_id/...
        return source.split("/")[3]


def detect_overspent_cpu(num_cpu: int, real_id: str, fm_step: FMStepSnapshot) -> str:
    """Produces a message warning about misconfiguration of NUM_CPU if
    so is detected. Returns an empty string if everything is ok."""
    allowed_overspending = 1.05
    now = datetime.datetime.now()
    duration = (
        (fm_step.get(ids.END_TIME) or now) - (fm_step.get(ids.START_TIME) or now)
    ).total_seconds()
    if duration <= 0:
        return ""
    cpu_seconds = fm_step.get(ids.CPU_SECONDS) or 0.0
    parallelization_obtained = cpu_seconds / duration
    if parallelization_obtained > num_cpu * allowed_overspending:
        return (
            f"Misconfigured NUM_CPU, forward model step '{fm_step.get(ids.NAME)}' for "
            f"realization {real_id} spent {cpu_seconds} cpu seconds "
            f"with wall clock duration {duration:.1f} seconds, "
            f"a factor of {parallelization_obtained:.2f}, while NUM_CPU was {num_cpu}."
        )
    return ""
