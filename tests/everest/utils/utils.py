import contextlib
import importlib.util
import os
import pathlib
import shutil
import sys
from io import StringIO
from pathlib import Path

import pytest

from everest.bin.main import start_everest
from everest.config import EverestConfig, ServerConfig
from everest.detached import ExperimentState, everserver_status
from everest.jobs import script_names


def skipif_no_everest_models(function):
    """Decorator to skip a test if everest-models is not available"""
    spec = importlib.util.find_spec("everest_models")
    not_found = spec is None
    return pytest.mark.skipif(not_found, reason="everest-models not found")(function)


def relpath(*path):
    return os.path.join(os.path.dirname(os.path.dirname(__file__)), *path)


@contextlib.contextmanager
def capture_streams():
    """Context that allows capturing text sent to stdout and stderr

    Use as follow:
    with capture_streams() as (out, err):
        foo()
    assert( 'output of foo' in out.getvalue())
    """
    new_out, new_err = StringIO(), StringIO()
    old_out, old_err = sys.stdout, sys.stderr
    try:
        sys.stdout, sys.stderr = new_out, new_err
        yield new_out, new_err
    finally:
        sys.stdout, sys.stderr = old_out, old_err


def satisfy(predicate):
    """Return a class that equals to an obj if predicate(obj) is True

    This method is expected to be used with `assert_called_with()` on mocks.
    An example can be found in `test_everest_entry.test_everest_run`
    Inspired by
    https://stackoverflow.com/questions/21611559/assert-that-a-method-was-called-with-one-argument-out-of-several
    """

    class _PredicateChecker:
        def __eq__(self, obj) -> bool:
            return predicate(obj)

    return _PredicateChecker()


def satisfy_type(the_type):
    """Specialization of satisfy for checking object type"""
    return satisfy(lambda obj: isinstance(obj, the_type))


def satisfy_callable():
    """Specialization of satisfy for checking that object is callable"""
    return satisfy(callable)


class MockParser:
    """
    Small class that contains the necessary functions in order to test custom
    validation functions used with the argparse module
    """

    def __init__(self) -> None:
        self.error_msg = None

    def get_error(self):
        return self.error_msg

    def error(self, value=None):
        self.error_msg = value


def everest_default_jobs(output_dir):
    return [
        (
            script_name,
            (
                os.path.join(output_dir, ".jobs", f"_{script_name}"),
                Path(os.path.join(output_dir, ".jobs", f"_{script_name}")).read_text(
                    encoding="utf-8"
                ),
            ),
        )
        for script_name in script_names
    ]


def create_cached_mocked_test_case(request, monkeypatch) -> pathlib.Path:
    """This function will run everest to create some mocked data,
    this is quite slow, but the results will be cached. If something comes
    out of sync, clear the cache and start again. (rm -fr .pytest_cache/)
    """
    config_file = "mocked_multi_batch.yml"
    config_path = relpath("test_data", "mocked_test_case")
    cache_path = request.config.cache.mkdir(
        "snake_oil_data" + os.environ.get("PYTEST_XDIST_WORKER", "")
    )
    if not os.path.exists(cache_path / "mocked_run"):
        monkeypatch.chdir(cache_path)
        shutil.copytree(config_path, "mocked_run")
        monkeypatch.chdir("mocked_run")
        start_everest(["everest", "run", config_file, "--skip-prompt"])
        config = EverestConfig.load_file(config_file)
        status = everserver_status(
            ServerConfig.get_everserver_status_path(config.output_dir)
        )
        assert status["status"] == ExperimentState.completed
    return cache_path / "mocked_run"
