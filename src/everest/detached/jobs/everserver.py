import argparse
import datetime
import json
import logging
import os
import socket
import ssl
import threading
import time
import traceback
from base64 import b64encode
from functools import partial
from pathlib import Path
from typing import Any

import requests
import uvicorn
from cryptography import x509
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.x509.oid import NameOID
from dns import resolver, reversename
from fastapi import Depends, FastAPI, HTTPException, Request, status
from fastapi.encoders import jsonable_encoder
from fastapi.responses import (
    JSONResponse,
    PlainTextResponse,
    Response,
)
from fastapi.security import (
    HTTPBasic,
    HTTPBasicCredentials,
)
from pydantic import BaseModel

from ert.config.parsing.queue_system import QueueSystem
from ert.ensemble_evaluator import EvaluatorServerConfig
from ert.run_models.everest_run_model import EverestExitCode, EverestRunModel
from everest import export_to_csv, export_with_progress
from everest.config import EverestConfig, ExportConfig, ServerConfig
from everest.detached import (
    PROXY,
    ServerStatus,
    get_opt_status,
    update_everserver_status,
    wait_for_server,
)
from everest.export import check_for_errors
from everest.plugins.everest_plugin_manager import EverestPluginManager
from everest.simulator import JOB_FAILURE
from everest.strings import (
    DEFAULT_LOGGING_FORMAT,
    EVEREST,
    EXPERIMENT_STATUS_ENDPOINT,
    OPT_FAILURE_REALIZATIONS,
    OPT_PROGRESS_ENDPOINT,
    SHARED_DATA_ENDPOINT,
    SIM_PROGRESS_ENDPOINT,
    START_EXPERIMENT_ENDPOINT,
    STOP_ENDPOINT,
)
from everest.util import makedirs_if_needed, version_info


class ExperimentStatus(BaseModel):
    exit_code: EverestExitCode
    message: str | None = None


class ExperimentRunner(threading.Thread):
    def __init__(self, everest_config, shared_data: dict):
        super().__init__()

        self._everest_config = everest_config
        self._shared_data = shared_data
        self._status: ExperimentStatus | None = None

    def run(self):
        run_model = EverestRunModel.create(
            self._everest_config,
            simulation_callback=partial(_sim_monitor, shared_data=self._shared_data),
            optimization_callback=partial(_opt_monitor, shared_data=self._shared_data),
        )

        if run_model._queue_config.queue_system == QueueSystem.LOCAL:
            evaluator_server_config = EvaluatorServerConfig()
        else:
            evaluator_server_config = EvaluatorServerConfig(
                custom_port_range=range(49152, 51819), use_ipc_protocol=False
            )

        try:
            run_model.run_experiment(evaluator_server_config)

            assert run_model.exit_code is not None
            self._status = ExperimentStatus(exit_code=run_model.exit_code)
        except Exception as e:
            self._status = ExperimentStatus(
                exit_code=EverestExitCode.EXCEPTION, message=str(e)
            )

    @property
    def status(self) -> ExperimentStatus | None:
        return self._status

    @property
    def shared_data(self) -> dict:
        return self._shared_data


def _get_machine_name() -> str:
    """Returns a name that can be used to identify this machine in a network

    A fully qualified domain name is returned if available. Otherwise returns
    the string `localhost`
    """
    hostname = socket.gethostname()
    try:
        # We need the ip-address to perform a reverse lookup to deal with
        # differences in how the clusters are getting their fqdn's
        ip_addr = socket.gethostbyname(hostname)
        reverse_name = reversename.from_address(ip_addr)
        resolved_hosts = [
            str(ptr_record).rstrip(".")
            for ptr_record in resolver.resolve(reverse_name, "PTR")
        ]
        resolved_hosts.sort()
        return resolved_hosts[0]
    except (resolver.NXDOMAIN, resolver.NoResolverConfiguration):
        # If local address and reverse lookup not working - fallback
        # to socket fqdn which are using /etc/hosts to retrieve this name
        return socket.getfqdn()
    except socket.gaierror:
        logging.debug(traceback.format_exc())
        return "localhost"


def _sim_monitor(context_status, shared_data=None):
    assert shared_data is not None

    status = context_status["status"]
    shared_data[SIM_PROGRESS_ENDPOINT] = {
        "batch_number": context_status["batch_number"],
        "status": {
            "running": status.get("Running", 0),
            "waiting": status.get("Waiting", 0),
            "pending": status.get("Pending", 0),
            "complete": status.get("Finished", 0),
            "failed": status.get("Failed", 0),
        },
        "progress": context_status["progress"],
    }

    if shared_data[STOP_ENDPOINT]:
        return "stop_queue"


def _opt_monitor(shared_data=None):
    assert shared_data is not None
    if shared_data[STOP_ENDPOINT]:
        return "stop_optimization"


def _everserver_thread(shared_data, server_config) -> None:
    app = FastAPI()
    security = HTTPBasic()

    runner: ExperimentRunner | None = None

    def _check_user(credentials: HTTPBasicCredentials) -> None:
        if credentials.password != server_config["authentication"]:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Invalid credentials",
                headers={"WWW-Authenticate": "Basic"},
            )

    def _log(request: Request) -> None:
        logging.getLogger("everserver").info(
            f"{request.scope['path']} entered from {request.client.host if request.client else 'unknown host'} with HTTP {request.method}"
        )

    @app.get("/")
    def get_status(
        request: Request, credentials: HTTPBasicCredentials = Depends(security)
    ) -> PlainTextResponse:
        _log(request)
        _check_user(credentials)
        return PlainTextResponse("Everest is running")

    @app.post("/" + STOP_ENDPOINT)
    def stop(
        request: Request, credentials: HTTPBasicCredentials = Depends(security)
    ) -> Response:
        _log(request)
        _check_user(credentials)
        shared_data[STOP_ENDPOINT] = True
        return Response("Raise STOP flag succeeded. Everest initiates shutdown..", 200)

    @app.get("/" + SIM_PROGRESS_ENDPOINT)
    def get_sim_progress(
        request: Request, credentials: HTTPBasicCredentials = Depends(security)
    ) -> JSONResponse:
        _log(request)
        _check_user(credentials)
        progress = shared_data[SIM_PROGRESS_ENDPOINT]
        return JSONResponse(jsonable_encoder(progress))

    @app.get("/" + OPT_PROGRESS_ENDPOINT)
    def get_opt_progress(
        request: Request, credentials: HTTPBasicCredentials = Depends(security)
    ) -> JSONResponse:
        _log(request)
        _check_user(credentials)
        progress = get_opt_status(server_config["optimization_output_dir"])
        return JSONResponse(jsonable_encoder(progress))

    @app.post("/" + START_EXPERIMENT_ENDPOINT)
    def start_experiment(
        config: EverestConfig,
        request: Request,
        credentials: HTTPBasicCredentials = Depends(security),
    ) -> Response:
        _log(request)
        _check_user(credentials)

        nonlocal runner
        if runner is None:
            runner = ExperimentRunner(config, shared_data)
            try:
                runner.start()
                return Response("Everest experiment started")
            except Exception as e:
                return Response(f"Could not start experiment: {e!s}", status_code=501)
        return Response("Everest experiment is running")

    @app.get("/" + EXPERIMENT_STATUS_ENDPOINT)
    def get_experiment_status(
        request: Request, credentials: HTTPBasicCredentials = Depends(security)
    ) -> Response:
        _log(request)
        _check_user(credentials)
        if shared_data[STOP_ENDPOINT]:
            return JSONResponse(
                ExperimentStatus(exit_code=EverestExitCode.USER_ABORT).model_dump_json()
            )
        if runner is None:
            return Response(None, 204)
        status = runner.status
        if status is None:
            return Response(None, 204)
        return JSONResponse(status.model_dump_json())

    @app.get("/" + SHARED_DATA_ENDPOINT)
    def get_shared_data(
        request: Request, credentials: HTTPBasicCredentials = Depends(security)
    ) -> JSONResponse:
        _log(request)
        _check_user(credentials)
        if runner is None:
            return JSONResponse(jsonable_encoder(shared_data))
        return JSONResponse(jsonable_encoder(runner.shared_data))

    uvicorn.run(
        app,
        host="0.0.0.0",
        port=server_config["port"],
        ssl_keyfile=server_config["key_path"],
        ssl_certfile=server_config["cert_path"],
        ssl_version=ssl.PROTOCOL_SSLv23,
        ssl_keyfile_password=server_config["key_passwd"],
        log_level=logging.CRITICAL,
    )


def _find_open_port(host, lower, upper) -> int:
    for port in range(lower, upper):
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.bind((host, port))
            sock.close()
            return port
        except OSError:
            logging.getLogger("everserver").info(
                f"Port {port} for host {host} is taken"
            )
    msg = f"No open port for host {host} in the range {lower}-{upper}"
    logging.getLogger("everserver").exception(msg)
    raise Exception(msg)


def _write_hostfile(host_file_path, host, port, cert, auth) -> None:
    if not os.path.exists(os.path.dirname(host_file_path)):
        os.makedirs(os.path.dirname(host_file_path))
    data = {
        "host": host,
        "port": port,
        "cert": cert,
        "auth": auth,
    }
    json_string = json.dumps(data)

    with open(host_file_path, "w", encoding="utf-8") as f:
        f.write(json_string)


def _configure_loggers(detached_dir: Path, log_dir: Path, logging_level: int) -> None:
    def make_handler_config(
        path: Path, log_level: str | int = "INFO"
    ) -> dict[str, Any]:
        makedirs_if_needed(path.parent)
        return {
            "class": "logging.FileHandler",
            "formatter": "default",
            "level": log_level,
            "filename": path,
        }

    logging_config = {
        "version": 1,
        "disable_existing_loggers": False,
        "handlers": {
            "root": {"level": "NOTSET", "class": "logging.NullHandler"},
            "everserver": make_handler_config(detached_dir / "endpoint.log"),
            "everest": make_handler_config(log_dir / "everest.log", logging_level),
            "forward_models": make_handler_config(
                log_dir / "forward_models.log", logging_level
            ),
        },
        "loggers": {
            "": {"handlers": ["root"], "level": "NOTSET"},
            "everserver": {"handlers": ["everserver"]},
            "everest": {"handlers": ["everest"]},
            "forward_models": {"handlers": ["forward_models"]},
        },
        "formatters": {
            "default": {"format": DEFAULT_LOGGING_FORMAT},
        },
    }

    logging.config.dictConfig(logging_config)
    EverestPluginManager().add_log_handle_to_root()


def main():
    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument("--config-file", type=str)
    arg_parser.add_argument("--debug", action="store_true")
    options = arg_parser.parse_args()
    config = EverestConfig.load_file(options.config_file)
    status_path = ServerConfig.get_everserver_status_path(config.output_dir)
    host_file = ServerConfig.get_hostfile_path(config.output_dir)

    try:
        _configure_loggers(
            detached_dir=Path(ServerConfig.get_detached_node_dir(config.output_dir)),
            log_dir=(
                Path(config.output_dir) / "logs"
                if config.log_dir is None
                else Path(config.log_dir)
            ),
            logging_level=config.logging_level if not options.debug else logging.DEBUG,
        )

        update_everserver_status(status_path, ServerStatus.starting)
        logging.getLogger(EVEREST).info(version_info())
        logging.getLogger(EVEREST).info(f"Output directory: {config.output_dir}")
        logging.getLogger(EVEREST).debug(str(options))

        authentication = _generate_authentication()
        cert_path, key_path, key_pw = _generate_certificate(
            ServerConfig.get_certificate_dir(config.output_dir)
        )
        host = _get_machine_name()
        port = _find_open_port(host, lower=5000, upper=5800)
        _write_hostfile(host_file, host, port, cert_path, authentication)

        shared_data = {
            SIM_PROGRESS_ENDPOINT: {},
            STOP_ENDPOINT: False,
        }

        server_config = {
            "optimization_output_dir": config.optimization_output_dir,
            "port": port,
            "cert_path": cert_path,
            "key_path": key_path,
            "key_passwd": key_pw,
            "authentication": authentication,
        }

        everserver_instance = threading.Thread(
            target=_everserver_thread,
            args=(shared_data, server_config),
        )
        everserver_instance.daemon = True
        everserver_instance.start()

    except:
        update_everserver_status(
            status_path,
            ServerStatus.failed,
            message=traceback.format_exc(),
        )
        return

    try:
        wait_for_server(config.output_dir, 60)

        update_everserver_status(status_path, ServerStatus.running)

        server_context = (ServerConfig.get_server_context(config.output_dir),)
        url, cert, auth = server_context[0]

        done = False
        experiment_status: ExperimentStatus | None = None
        # loop until the optimization is done
        while not done:
            response = requests.get(
                "/".join([url, EXPERIMENT_STATUS_ENDPOINT]),
                verify=cert,
                auth=auth,
                timeout=1,
                proxies=PROXY,  # type: ignore
            )
            if response.status_code == requests.codes.OK:
                json_body = json.loads(
                    response.text if hasattr(response, "text") else response.body
                )
                experiment_status = ExperimentStatus.model_validate_json(json_body)
                done = True
            else:
                time.sleep(1)

        response = requests.get(
            "/".join([url, SHARED_DATA_ENDPOINT]),
            verify=cert,
            auth=auth,
            timeout=1,
            proxies=PROXY,  # type: ignore
        )
        if json_body := json.loads(
            response.text if hasattr(response, "text") else response.body
        ):
            shared_data = json_body

        assert experiment_status is not None
        status, message = _get_optimization_status(experiment_status, shared_data)
        if status != ServerStatus.completed:
            update_everserver_status(status_path, status, message)
            return
    except:
        if shared_data[STOP_ENDPOINT]:
            update_everserver_status(
                status_path,
                ServerStatus.stopped,
                message="Optimization aborted.",
            )
        else:
            update_everserver_status(
                status_path,
                ServerStatus.failed,
                message=traceback.format_exc(),
            )
        return

    try:
        # Exporting data
        update_everserver_status(status_path, ServerStatus.exporting_to_csv)

        if config.export is not None:
            err_msgs, export_ecl = check_for_errors(
                config=config.export,
                optimization_output_path=config.optimization_output_dir,
                storage_path=config.storage_dir,
                data_file_path=config.model.data_file,
            )
            for msg in err_msgs:
                logging.getLogger(EVEREST).warning(msg)
        else:
            export_ecl = True
        export_config = config.export or ExportConfig()
        export_to_csv(
            data_frame=export_with_progress(config, export_config, export_ecl),
            export_path=config.export_path,
        )
    except:
        update_everserver_status(
            status_path,
            ServerStatus.failed,
            message=traceback.format_exc(),
        )
        return

    update_everserver_status(status_path, ServerStatus.completed, message=message)


def _get_optimization_status(
    experiment_status: ExperimentStatus, shared_data: dict
) -> tuple[ServerStatus, str]:
    match experiment_status.exit_code:
        case EverestExitCode.MAX_BATCH_NUM_REACHED:
            return ServerStatus.completed, "Maximum number of batches reached."

        case EverestExitCode.MAX_FUNCTIONS_REACHED:
            return (
                ServerStatus.completed,
                "Maximum number of function evaluations reached.",
            )

        case EverestExitCode.USER_ABORT:
            return ServerStatus.stopped, "Optimization aborted."

        case EverestExitCode.EXCEPTION:
            assert experiment_status.message is not None
            return ServerStatus.failed, experiment_status.message

        case EverestExitCode.TOO_FEW_REALIZATIONS:
            status = (
                ServerStatus.stopped
                if shared_data[STOP_ENDPOINT]
                else ServerStatus.failed
            )
            messages = _failed_realizations_messages(shared_data)
            for msg in messages:
                logging.getLogger(EVEREST).error(msg)
            return status, "\n".join(messages)
        case _:
            return ServerStatus.completed, "Optimization completed."


def _failed_realizations_messages(shared_data):
    messages = [OPT_FAILURE_REALIZATIONS]
    failed = shared_data[SIM_PROGRESS_ENDPOINT]["status"]["failed"]
    if failed > 0:
        # Report each unique pair of failed job name and error
        for queue in shared_data[SIM_PROGRESS_ENDPOINT]["progress"]:
            for job in queue:
                if job["status"] == JOB_FAILURE:
                    err_msg = f"{job['name']} Failed with: {job.get('error', '')}"
                    if err_msg not in messages:
                        messages.append(err_msg)
    return messages


def _generate_certificate(cert_folder: str):
    """Generate a private key and a certificate signed with it

    Both the certificate and the key are written to files in the folder given
    by `get_certificate_dir(config)`. The key is encrypted before being
    stored.
    Returns the path to the certificate file, the path to the key file, and
    the password used for encrypting the key
    """
    # Generate private key
    key = rsa.generate_private_key(
        public_exponent=65537, key_size=4096, backend=default_backend()
    )

    # Generate the certificate and sign it with the private key
    cert_name = _get_machine_name()
    subject = issuer = x509.Name(
        [
            x509.NameAttribute(NameOID.COUNTRY_NAME, "NO"),
            x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, "Bergen"),
            x509.NameAttribute(NameOID.LOCALITY_NAME, "Sandsli"),
            x509.NameAttribute(NameOID.ORGANIZATION_NAME, "Equinor"),
            x509.NameAttribute(NameOID.COMMON_NAME, f"{cert_name}"),
        ]
    )
    cert = (
        x509.CertificateBuilder()
        .subject_name(subject)
        .issuer_name(issuer)
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(datetime.datetime.now(datetime.UTC))
        .not_valid_after(
            datetime.datetime.now(datetime.UTC) + datetime.timedelta(days=365)
        )  # 1 year
        .add_extension(
            x509.SubjectAlternativeName([x509.DNSName(f"{cert_name}")]),
            critical=False,
        )
        .sign(key, hashes.SHA256(), default_backend())
    )

    # Write certificate and key to disk
    makedirs_if_needed(cert_folder)
    cert_path = os.path.join(cert_folder, cert_name + ".crt")
    with open(cert_path, "wb") as f:
        f.write(cert.public_bytes(serialization.Encoding.PEM))
    key_path = os.path.join(cert_folder, cert_name + ".key")
    pw = bytes(os.urandom(28))
    with open(key_path, "wb") as f:
        f.write(
            key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.TraditionalOpenSSL,
                encryption_algorithm=serialization.BestAvailableEncryption(pw),
            )
        )
    return cert_path, key_path, pw


def _generate_authentication():
    n_bytes = 128
    random_bytes = bytes(os.urandom(n_bytes))
    return b64encode(random_bytes).decode("utf-8")
