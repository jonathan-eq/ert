import json
import os
import shutil
import stat
import tempfile
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from pathlib import Path
from textwrap import dedent
from typing import Any
from unittest.mock import MagicMock, PropertyMock, patch
from uuid import UUID

import hypothesis.strategies as st
import numpy as np
import orjson
import polars as pl
import pytest
import xarray as xr
from hypothesis import assume, given, note, settings
from hypothesis.extra.numpy import arrays
from hypothesis.stateful import Bundle, RuleBasedStateMachine, initialize, rule
from pandas import DataFrame, ExcelWriter

from ert.config import (
    DESIGN_MATRIX_GROUP,
    DesignMatrix,
    EnkfObs,
    ErtConfig,
    ExtParamConfig,
    Field,
    GenDataConfig,
    GenKwConfig,
    ObservationType,
    ParameterConfig,
    ResponseConfig,
    SummaryConfig,
    SurfaceConfig,
)
from ert.config.gen_kw_config import TransformFunctionDefinition
from ert.config.general_observation import GenObservation
from ert.config.observation_vector import ObsVector
from ert.enkf_main import sample_prior, save_design_matrix_to_ensemble
from ert.storage import ErtStorageException, LocalEnsemble, open_storage
from ert.storage.local_storage import _LOCAL_STORAGE_VERSION
from ert.storage.mode import ModeError
from ert.storage.realization_storage_state import RealizationStorageState
from tests.ert.unit_tests.config.egrid_generator import egrids
from tests.ert.unit_tests.config.summary_generator import summaries, summary_variables


def _ensembles(storage):
    return sorted(x.name for x in storage.ensembles)


def test_create_experiment(tmp_path):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment(name="test-experiment")

        experiment_path = Path(storage.path / "experiments" / str(experiment.id))
        assert experiment_path.exists()

        assert (experiment_path / experiment._parameter_file).exists()
        assert (experiment_path / experiment._responses_file).exists()

        with open(experiment_path / "index.json", encoding="utf-8") as f:
            index = json.load(f)
            assert index["id"] == str(experiment.id)
            assert index["name"] == "test-experiment"


def test_that_loading_non_existing_experiment_throws(tmp_path):
    with (
        open_storage(tmp_path, mode="w") as storage,
        pytest.raises(
            KeyError, match="Experiment with name 'non-existing-experiment' not found"
        ),
    ):
        storage.get_experiment_by_name("non-existing-experiment")


def test_that_loading_non_existing_ensemble_throws(tmp_path):
    with (
        open_storage(tmp_path, mode="w") as storage,
        pytest.raises(
            KeyError, match="Ensemble with name 'non-existing-ensemble' not found"
        ),
    ):
        experiment = storage.create_experiment(name="test-experiment")
        experiment.get_ensemble_by_name("non-existing-ensemble")


def test_that_saving_empty_responses_fails_nicely(tmp_path):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment()
        ensemble = storage.create_ensemble(
            experiment, ensemble_size=1, iteration=0, name="prior"
        )

        # Test for entirely empty dataset
        with pytest.raises(
            ValueError,
            match=(
                "Dataset for response group 'RESPONSE' must contain a 'values' variable"
            ),
        ):
            ensemble.save_response("RESPONSE", pl.DataFrame(), 0)

        # Test for dataset with 'values' but no actual data
        empty_data = pl.DataFrame(
            {
                "response_key": [],
                "report_step": [],
                "index": [],
                "values": [],
            }
        )

        with pytest.raises(
            ValueError,
            match=(
                "Responses RESPONSE are empty\\. "
                "Cannot proceed with saving to storage\\."
            ),
        ):
            ensemble.save_response("RESPONSE", empty_data, 0)


def test_that_saving_response_updates_configs(tmp_path):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment(
            responses=[SummaryConfig(keys=["*", "FOPR"], input_files=["not_relevant"])]
        )
        ensemble = storage.create_ensemble(
            experiment, ensemble_size=1, iteration=0, name="prior"
        )

        summary_df = pl.DataFrame(
            {
                "response_key": ["FOPR", "FOPT:OP1", "FOPR:OP3", "FLAP", "F*"],
                "time": pl.Series(
                    [datetime(2000, 1, i) for i in range(1, 6)]
                ).dt.cast_time_unit("ms"),
                "values": pl.Series([0.0, 1.0, 2.0, 3.0, 4.0], dtype=pl.Float32),
            }
        )

        mapping_before = experiment.response_key_to_response_type
        smry_config_before = experiment.response_configuration["summary"]

        assert not ensemble.experiment._has_finalized_response_keys("summary")
        ensemble.save_response("summary", summary_df, 0)

        assert ensemble.experiment._has_finalized_response_keys("summary")
        assert ensemble.experiment.response_key_to_response_type == {
            "FOPR:OP3": "summary",
            "F*": "summary",
            "FLAP": "summary",
            "FOPR": "summary",
            "FOPT:OP1": "summary",
        }
        assert ensemble.experiment.response_type_to_response_keys == {
            "summary": ["F*", "FLAP", "FOPR", "FOPR:OP3", "FOPT:OP1"]
        }

        mapping_after = experiment.response_key_to_response_type
        smry_config_after = experiment.response_configuration["summary"]

        assert set(mapping_before) == set()
        assert set(smry_config_before.keys) == {"*", "FOPR"}

        assert set(mapping_after) == {"F*", "FOPR", "FOPT:OP1", "FOPR:OP3", "FLAP"}
        assert set(smry_config_after.keys) == {
            "FOPR",
            "FOPT:OP1",
            "FOPR:OP3",
            "FLAP",
            "F*",
        }


def test_that_saving_empty_parameters_fails_nicely(tmp_path):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment()
        prior = storage.create_ensemble(
            experiment, ensemble_size=1, iteration=0, name="prior"
        )

        # Test for entirely empty dataset
        with pytest.raises(
            ValueError,
            match=(
                "Dataset for parameter group 'PARAMETER' "
                "must contain a 'values' variable"
            ),
        ):
            prior.save_parameters("PARAMETER", 0, xr.Dataset())

        # Test for dataset with 'values' and 'transformed_values' but no actual data
        empty_data = xr.Dataset(
            {
                "values": ("names", np.array([], dtype=float)),
                "names": (["names"], np.array([], dtype=str)),
            }
        )
        with pytest.raises(
            ValueError,
            match=(
                "Parameters PARAMETER are empty\\. "
                "Cannot proceed with saving to storage\\."
            ),
        ):
            prior.save_parameters("PARAMETER", 0, empty_data)


def test_that_loading_parameter_via_response_api_fails(tmp_path):
    uniform_parameter = GenKwConfig(
        name="PARAMETER",
        forward_init=False,
        transform_function_definitions=[
            TransformFunctionDefinition("KEY1", "UNIFORM", [0, 1]),
        ],
        update=True,
    )
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment(
            parameters=[uniform_parameter],
        )
        prior = storage.create_ensemble(
            experiment,
            ensemble_size=1,
            iteration=0,
            name="prior",
        )

        prior.save_parameters(
            "PARAMETER",
            0,
            xr.Dataset(
                {
                    "values": ("names", [1.0]),
                    "names": ["KEY_1"],
                }
            ),
        )
        with pytest.raises(ValueError, match="PARAMETER is not a response"):
            prior.load_responses("PARAMETER", (0,))


def test_that_load_responses_throws_exception(tmp_path):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment()
        ensemble = storage.create_ensemble(experiment, name="foo", ensemble_size=1)

        with pytest.raises(
            expected_exception=ValueError, match="I_DONT_EXIST is not a response"
        ):
            ensemble.load_responses("I_DONT_EXIST", (1,))


def test_that_load_parameters_throws_exception(tmp_path):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment()
        ensemble = storage.create_ensemble(experiment, name="foo", ensemble_size=1)

        with pytest.raises(expected_exception=KeyError):
            ensemble.load_parameters("I_DONT_EXIST", 1)


def test_open_empty_read(tmp_path):
    with open_storage(tmp_path / "empty", mode="r") as storage:
        assert _ensembles(storage) == []

    # Storage doesn't create an empty directory
    assert not (tmp_path / "empty").is_dir()


def test_open_empty_write(tmp_path):
    with open_storage(tmp_path / "empty", mode="w") as storage:
        assert _ensembles(storage) == []

    # Storage creates the directory
    assert (tmp_path / "empty").is_dir()


def test_refresh(tmp_path):
    with open_storage(tmp_path, mode="w") as accessor:
        experiment_id = accessor.create_experiment()
        with open_storage(tmp_path, mode="r") as reader:
            assert _ensembles(accessor) == _ensembles(reader)

            accessor.create_ensemble(experiment_id, name="foo", ensemble_size=42)
            # Reader does not know about the newly created ensemble
            assert _ensembles(accessor) != _ensembles(reader)

            reader.refresh()
            # Reader knows about it after the refresh
            assert _ensembles(accessor) == _ensembles(reader)


def test_that_reader_storage_reads_most_recent_response_configs(tmp_path):
    reader = open_storage(tmp_path, mode="r")
    writer = open_storage(tmp_path, mode="w")

    exp = writer.create_experiment(
        responses=[SummaryConfig(keys=["*", "FOPR"], input_files=["not_relevant"])],
        name="uniq",
    )
    ens: LocalEnsemble = exp.create_ensemble(ensemble_size=10, name="uniq_ens")

    reader.refresh()
    read_exp = reader.get_experiment_by_name("uniq")
    assert read_exp.id == exp.id

    read_smry_config = read_exp.response_configuration["summary"]
    assert read_smry_config.keys == ["*", "FOPR"]
    assert not read_smry_config.has_finalized_keys

    smry_data = pl.DataFrame(
        {
            "response_key": ["FOPR", "FOPR", "WOPR", "WOPR", "FOPT", "FOPT"],
            "time": pl.Series(
                [datetime.now() + timedelta(days=i) for i in range(6)]
            ).dt.cast_time_unit("ms"),
            "values": pl.Series([0.2, 0.2, 1.0, 1.1, 3.3, 3.3], dtype=pl.Float32),
        }
    )

    ens.save_response("summary", smry_data, 0)
    assert read_smry_config.keys == ["*", "FOPR"]
    assert not read_smry_config.has_finalized_keys

    read_smry_config = read_exp.response_configuration["summary"]
    assert read_smry_config.keys == ["FOPR", "FOPT", "WOPR"]
    assert read_smry_config.has_finalized_keys


def test_writing_to_read_only_storage_raises(tmp_path):
    with open_storage(tmp_path, mode="r") as storage, pytest.raises(ModeError):
        storage.create_experiment()


@pytest.mark.timeout(10)
def test_open_storage_write_with_empty_directory(tmp_path, caplog):
    with open_storage(tmp_path / "storage", mode="w") as storage:
        _ = storage.create_experiment()
        assert len(list(storage.experiments)) == 1

    with open_storage(tmp_path / "storage", mode="w") as storage:
        _ = storage.create_experiment()
        assert len(list(storage.experiments)) == 1

    storage.refresh()
    assert len(list(storage.experiments)) == 0

    assert len(caplog.messages) == 0


def test_open_storage_read_with_empty_directory(tmp_path, caplog):
    with open_storage(tmp_path / "storage", mode="r") as storage:
        assert list(storage.ensembles) == []
        assert list(storage.experiments) == []


def test_open_storage_nested_dirs(tmp_path, caplog):
    with open_storage(tmp_path / "extra_level" / "storage", mode="w") as storage:
        assert storage.path.exists()


def test_open_storage_with_corrupted_storage(tmp_path):
    with open_storage(tmp_path / "storage", mode="w") as storage:
        storage.create_experiment().create_ensemble(name="prior", ensemble_size=1)
    os.remove(tmp_path / "storage" / "index.json")
    with pytest.raises(ErtStorageException, match="No index\\.json"):
        open_storage(tmp_path / "storage", mode="w")


def test_that_open_storage_in_read_mode_with_newer_version_throws_exception(
    tmp_path, caplog
):
    with open_storage(tmp_path, mode="w") as storage:
        storage._index.version = _LOCAL_STORAGE_VERSION + 1
        storage._save_index()

    with pytest.raises(
        ErtStorageException,
        match=(
            f"Cannot open storage '{tmp_path}': "
            f"Storage version {_LOCAL_STORAGE_VERSION + 1} is newer than "
            f"the current version {_LOCAL_STORAGE_VERSION}, upgrade ert "
            "to continue, or run with a different ENSPATH"
        ),
    ):
        open_storage(tmp_path, mode="r")


def test_that_open_storage_in_read_mode_with_older_version_throws_exception(
    tmp_path, caplog
):
    with open_storage(tmp_path, mode="w") as storage:
        storage._index.version = _LOCAL_STORAGE_VERSION - 1
        storage._save_index()

    with pytest.raises(
        ErtStorageException,
        match=(
            f"Cannot open storage '{tmp_path}' in read-only mode: "
            f"Storage version {_LOCAL_STORAGE_VERSION - 1} is too old"
        ),
    ):
        open_storage(tmp_path, mode="r")


def test_that_open_storage_in_write_mode_with_newer_version_throws_exception(
    tmp_path, caplog
):
    with open_storage(tmp_path, mode="w") as storage:
        storage._index.version = _LOCAL_STORAGE_VERSION + 1
        storage._save_index()

    with pytest.raises(
        ErtStorageException,
        match=(
            f"Cannot open storage '{tmp_path}': "
            f"Storage version {_LOCAL_STORAGE_VERSION + 1} "
            f"is newer than the current version {_LOCAL_STORAGE_VERSION}, "
            "upgrade ert to continue, or run with a different ENSPATH"
        ),
    ):
        open_storage(tmp_path, mode="w")


def test_ensemble_no_parameters(storage):
    ensemble = storage.create_experiment(name="my-experiment").create_ensemble(
        ensemble_size=2,
        name="prior",
    )
    assert all(
        RealizationStorageState.RESPONSES_LOADED in s
        for s in ensemble.get_ensemble_state()
    )


def test_get_unique_experiment_name(snake_oil_storage):
    with patch(
        "ert.storage.local_storage.LocalStorage.experiments", new_callable=PropertyMock
    ) as experiments:
        # Its not possible to do MagicMock(name="experiment_name") therefore
        # the workaround below
        names = [
            "experiment",
            "experiment_1",
            "experiment_8",
            "_d_e_",
            "___name__0___",
            "__name__1",
            "default",
        ]
        experiment_list = [MagicMock() for _ in range(len(names))]
        for k, v in zip(experiment_list, names, strict=False):
            k.name = v
        experiments.return_value = experiment_list

        assert snake_oil_storage.get_unique_experiment_name("_d_e_") == "_d_e__0"
        assert (
            snake_oil_storage.get_unique_experiment_name("experiment") == "experiment_9"
        )
        assert (
            snake_oil_storage.get_unique_experiment_name("___name__0___")
            == "___name__0____0"
        )
        assert snake_oil_storage.get_unique_experiment_name("name") == "name"
        assert snake_oil_storage.get_unique_experiment_name("__name__") == "__name__"
        assert snake_oil_storage.get_unique_experiment_name("") == "default_0"


def add_to_name(prefix: str):
    def _inner(params):
        for param in params:
            param.name = prefix + param.name
        return params

    return _inner


parameter_configs = st.lists(
    st.one_of(
        st.builds(
            GenKwConfig,
            name=st.text(),
            update=st.booleans(),
            forward_init=st.booleans(),
            transform_function_definitions=st.just([]),
        ),
        st.builds(SurfaceConfig),
    ),
    unique_by=lambda x: x.name,
    min_size=1,
).map(add_to_name("parameter_"))

summary_selectors = st.one_of(
    summary_variables(), st.just("*"), summary_variables().map(lambda x: x + "*")
)

response_configs = st.lists(
    st.one_of(
        st.builds(
            GenDataConfig,
        ),
        st.builds(
            SummaryConfig,
            name=st.just("summary"),
            input_files=st.lists(
                st.text(
                    alphabet=st.characters(
                        min_codepoint=ord("A"), max_codepoint=ord("Z")
                    )
                ),
                min_size=1,
                max_size=1,
            ),
            keys=st.lists(summary_selectors, min_size=1),
        ),
    ),
    unique_by=lambda x: x.name,
    min_size=1,
)

ensemble_sizes = st.integers(min_value=1, max_value=1000)
coordinates = st.integers(min_value=1, max_value=100)

words = st.text(
    min_size=1,
    max_size=8,
    alphabet=st.characters(min_codepoint=ord("A"), max_codepoint=ord("Z")),
)

gen_observations = st.integers(min_value=1, max_value=10).flatmap(
    lambda size: st.builds(
        GenObservation,
        values=arrays(np.double, shape=size),
        stds=arrays(
            np.double,
            elements=st.floats(min_value=0.1, max_value=1.0),
            shape=size,
        ),
        indices=arrays(
            np.int64,
            elements=st.integers(min_value=0, max_value=100),
            shape=size,
        ),
        std_scaling=arrays(np.double, shape=size),
    )
)

observations = st.builds(
    EnkfObs,
    obs_vectors=st.dictionaries(
        words,
        st.builds(
            ObsVector,
            observation_type=st.just(ObservationType.GENERAL),
            observation_key=words,
            data_key=words,
            observations=st.dictionaries(
                st.integers(min_value=0, max_value=200),
                gen_observations,
                min_size=1,
                max_size=1,
            ),
        ),
    ),
    obs_time=st.lists(
        st.datetimes(
            min_value=datetime.strptime("1969-1-1", "%Y-%m-%d"),
            max_value=datetime.strptime("3000-1-1", "%Y-%m-%d"),
        )
    ),
)

small_ints = st.integers(min_value=1, max_value=10)


@st.composite
def fields(draw, egrid, num_fields=small_ints) -> list[Field]:
    grid_file, grid = egrid
    nx, ny, nz = grid.shape
    return [
        draw(
            st.builds(
                Field,
                name=st.just(f"Field{i}"),
                file_format=st.just("roff_binary"),
                grid_file=st.just(grid_file),
                nx=st.just(nx),
                ny=st.just(ny),
                nz=st.just(nz),
                output_file=st.just(Path(f"field{i}.roff")),
            )
        )
        for i in range(draw(num_fields))
    ]


@pytest.mark.usefixtures("use_tmpdir")
@given(st.binary())
def test_write_transaction(data):
    with open_storage(".", "w") as storage:
        filepath = Path("./file.txt")
        storage._write_transaction(filepath, data)

        assert stat.S_IMODE(filepath.stat().st_mode) == 0o660
        assert filepath.read_bytes() == data


class RaisingWriteNamedTemporaryFile:
    entered = False

    def __init__(self, *args, **kwargs) -> None:
        self.wrapped = tempfile.NamedTemporaryFile(*args, **kwargs)  # noqa
        RaisingWriteNamedTemporaryFile.entered = False

    def __enter__(self, *args, **kwargs) -> MagicMock:
        self.actual_handle = self.wrapped.__enter__(*args, **kwargs)
        mock_handle = MagicMock()
        RaisingWriteNamedTemporaryFile.entered = True

        def ctrlc(_):
            raise RuntimeError

        mock_handle.write = ctrlc
        return mock_handle

    def __exit__(self, *args, **kwargs) -> None:
        self.wrapped.__exit__(*args, **kwargs)


def test_write_transaction_failure(tmp_path):
    with open_storage(tmp_path, "w") as storage:
        path = tmp_path / "file.txt"
        with (
            patch(
                "ert.storage.local_storage.NamedTemporaryFile",
                RaisingWriteNamedTemporaryFile,
            ) as f,
            pytest.raises(RuntimeError),
        ):
            storage._write_transaction(path, b"deadbeaf")

        assert f.entered

        assert not path.exists()


def test_write_transaction_overwrites(tmp_path):
    with open_storage(tmp_path, "w") as storage:
        path = tmp_path / "file.txt"
        path.write_text("abc")
        storage._write_transaction(path, b"deadbeaf")
        assert path.read_bytes() == b"deadbeaf"


@pytest.mark.parametrize(
    "perturb_observations, perturb_responses",
    [
        pytest.param(
            False,
            True,
            id="Perturbed responses",
        ),
        pytest.param(
            True,
            False,
            id="Perturbed observations",
        ),
        pytest.param(
            True,
            False,
            id="Perturbed observations & responses",
        ),
    ],
)
def test_asof_joining_summary(tmp_path, perturb_observations, perturb_responses):
    with open_storage(tmp_path, mode="w") as storage:
        response_keys = ["FOPR", "FOPT_OP1", "FOPR:OP3", "FLAP", "F*"]
        obs_keys = [f"o_{k}" for k in response_keys]
        times = [datetime(2000, 1, 1, 1, 0)] * len(response_keys)
        summary_observations = pl.DataFrame(
            {
                "observation_key": obs_keys,
                "response_key": response_keys,
                "time": pl.Series(
                    times,
                    dtype=pl.Datetime("ms"),
                ),
                "observations": pl.Series(
                    [1] * len(response_keys),
                    dtype=pl.Float32,
                ),
                "std": pl.Series(
                    [0.1] * len(response_keys),
                    dtype=pl.Float32,
                ),
            }
        )

        experiment = storage.create_experiment(
            responses=[SummaryConfig(keys=["*"], input_files=["not_relevant"])],
            observations={"summary": summary_observations},
        )

        ensemble = storage.create_ensemble(
            experiment, ensemble_size=1, iteration=0, name="prior"
        )

        summary_df = pl.DataFrame(
            {
                "response_key": response_keys,
                "time": pl.Series(times, dtype=pl.Datetime("ms")),
                "values": pl.Series([0.0, 1.0, 2.0, 3.0, 4.0], dtype=pl.Float32),
            }
        )

        ensemble.save_response("summary", summary_df, 0)
        iens_active_index = np.array([0])

        obs_and_responses_exact = ensemble.get_observations_and_responses(
            obs_keys, iens_active_index
        )

        rng = np.random.default_rng(12345678)

        if perturb_responses:
            perturbed_summary = summary_df.with_columns(
                pl.when(pl.arange(0, summary_df.height) % 2 != 0)
                .then(pl.col("time") + pl.duration(milliseconds=rng.random() * 500))
                .otherwise(
                    pl.col("time") - pl.duration(milliseconds=rng.random() * 500)
                )
                .alias("time")
            )
            perturbed_summary = perturbed_summary.sort(by="time")
            ensemble.save_response("summary", perturbed_summary, 0)

        if perturb_observations:
            perturbed_observations = summary_observations.with_columns(
                pl.when(pl.arange(0, summary_observations.height) % 2 != 0)
                .then(pl.col("time") + pl.duration(milliseconds=rng.random() * 500))
                .otherwise(
                    pl.col("time") - pl.duration(milliseconds=rng.random() * 500)
                )
                .alias("time")
            )
            perturbed_observations = perturbed_observations.sort(by="time")
            experiment.observations["summary"] = perturbed_observations

        obs_and_responses_perturbed = ensemble.get_observations_and_responses(
            obs_keys, iens_active_index
        )

        assert (
            obs_and_responses_exact.sort("response_key")
            .drop("index")
            .equals(obs_and_responses_perturbed.sort("response_key").drop("index"))
        )


def test_saving_everest_metadata_to_ensemble(tmp_path):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment(
            responses=[],
        )

        ensemble = storage.create_ensemble(
            experiment, ensemble_size=10, iteration=0, name="prior"
        )

        assert ensemble.everest_realization_info is None

        realization_info_dict = {
            ert_realization: {"model_realization": 0, "perturbation": -1}
            for ert_realization in range(10)
        }

        ensemble.save_everest_realization_info(realization_info_dict)
        assert ensemble.everest_realization_info == realization_info_dict


def test_that_saving_partial_everest_realization_info_raises_error(tmp_path):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment()

        ensemble = storage.create_ensemble(
            experiment, ensemble_size=10, iteration=0, name="prior"
        )

        with pytest.raises(
            ValueError,
            match=r"Everest realization info must describe all realizations.*[0, 2].*",
        ):
            ensemble.save_everest_realization_info({0: {}, 2: {}})


@pytest.mark.parametrize(
    "bad_realization_info",
    [
        {
            ert_realization: {"model_realization": None, "perturbation": -1}
            for ert_realization in range(10)
        },
        {
            ert_realization: {"model_realization": 0, "perturbation": None}
            for ert_realization in range(10)
        },
        {
            ert_realization: {"model_realization": 0, "perturbation": -2}
            for ert_realization in range(10)
        },
    ],
)
def test_that_saving_invalid_everest_realization_info_raises_error(
    tmp_path, bad_realization_info
):
    with open_storage(tmp_path, mode="w") as storage:
        experiment = storage.create_experiment()

        ensemble = storage.create_ensemble(
            experiment, ensemble_size=10, iteration=0, name="prior"
        )

        with pytest.raises(ValueError, match="Bad everest realization info"):
            ensemble.save_everest_realization_info(bad_realization_info)


_ensemble_realization_infos = [
    [
        {"model_realization": 0, "perturbation": -1},
        {"model_realization": 5, "perturbation": -1},
    ],
    [
        {"model_realization": 0, "perturbation": 0},
        {"model_realization": 0, "perturbation": 1},
        {"model_realization": 0, "perturbation": 2},
        {"model_realization": 5, "perturbation": 0},
        {"model_realization": 5, "perturbation": 1},
        {"model_realization": 5, "perturbation": 2},
    ],
    [
        {"model_realization": 0, "perturbation": -1},
        {"model_realization": 5, "perturbation": -1},
        {"model_realization": 0, "perturbation": 0},
        {"model_realization": 0, "perturbation": 1},
        {"model_realization": 0, "perturbation": 2},
        {"model_realization": 5, "perturbation": 0},
        {"model_realization": 5, "perturbation": 1},
        {"model_realization": 5, "perturbation": 2},
    ],
]


@pytest.mark.parametrize(
    "ensemble_realization_infos, failed_realizations_per_batch",
    [
        (_ensemble_realization_infos, {}),
        (_ensemble_realization_infos, {0: {0}, 1: {3}, 2: {0, 1, 4}}),
    ],
)
def test_that_all_parameters_and_gen_data_consolidation_works(
    ensemble_realization_infos, failed_realizations_per_batch, tmp_path, snapshot
):
    with open_storage(tmp_path, mode="w") as storage:
        param_keys = ["P1", "P2"]
        response_keys = ["R1", "R2"]

        experiment = storage.create_experiment(
            responses=[GenDataConfig(keys=["R1", "R2"])],
            parameters=[ExtParamConfig(name="point", input_keys=["P1", "P2"])],
        )

        ensemble_datas = []
        for batch, realization_info in enumerate(ensemble_realization_infos):
            failed_realizations = failed_realizations_per_batch.get(batch, {})
            num_realizations = len(realization_info)
            everest_realization_info = {i: v for i, v in enumerate(realization_info)}  # noqa: C416
            ensemble = storage.create_ensemble(
                experiment, ensemble_size=num_realizations, iteration=batch
            )

            ensemble.save_everest_realization_info(everest_realization_info)

            for realization in range(num_realizations):
                param_data = xr.Dataset(
                    {
                        "values": (
                            "names",
                            np.array([realization] * len(param_keys)) + (batch / 10),
                        ),
                        "names": param_keys,
                    }
                )
                ensemble.save_parameters("point", realization, param_data)

                if realization in failed_realizations:
                    ensemble.set_failure(
                        realization,
                        RealizationStorageState.LOAD_FAILURE,
                        "Failed to load responses",
                    )
                else:
                    response_data = pl.DataFrame(
                        {
                            "response_key": response_keys,
                            "values": np.array([realization * 10] * len(response_keys))
                            + (batch / 10),
                            "index": 0,
                            "report_step": 0,
                        }
                    )

                    ensemble.save_response("gen_data", response_data, realization)

            ensemble_data = ensemble.all_parameters_and_gen_data
            snapshot_str = (
                orjson.dumps(
                    ensemble_data.to_dicts(),
                    option=orjson.OPT_INDENT_2 | orjson.OPT_SORT_KEYS,
                )
                .decode("utf-8")
                .strip()
                + "\n"
            )
            ensemble_datas.append(ensemble_data)

            snapshot.assert_match(snapshot_str, f"batch_{batch}.json")

        experiment_data = experiment.all_parameters_and_gen_data
        snapshot_str = (
            orjson.dumps(
                experiment_data.to_dicts(),
                option=orjson.OPT_INDENT_2 | orjson.OPT_SORT_KEYS,
            )
            .decode("utf-8")
            .strip()
            + "\n"
        )
        snapshot.assert_match(snapshot_str, "all_batches.json")

        assert pl.concat(ensemble_datas).equals(experiment_data)


@pytest.mark.parametrize(
    "reals, expect_error",
    [
        pytest.param(
            list(range(10)),
            False,
            id="correct_active_realizations",
        ),
        pytest.param([10, 11], True, id="incorrect_active_realizations"),
    ],
)
def test_save_parameters_to_storage_from_design_dataframe(
    tmp_path, reals, expect_error
):
    design_path = tmp_path / "design_matrix.xlsx"
    ensemble_size = 10
    a_values = np.random.default_rng().uniform(-5, 5, 10)
    b_values = np.random.default_rng().uniform(-5, 5, 10)
    c_values = np.random.default_rng().uniform(-5, 5, 10)
    design_matrix_df = DataFrame({"a": a_values, "b": b_values, "c": c_values})
    with ExcelWriter(design_path) as xl_write:
        design_matrix_df.to_excel(xl_write, index=False, sheet_name="DesignSheet")
        DataFrame().to_excel(
            xl_write, index=False, sheet_name="DefaultSheet", header=False
        )
    design_matrix = DesignMatrix(design_path, "DesignSheet", "DefaultSheet")
    with open_storage(tmp_path / "storage", mode="w") as storage:
        experiment_id = storage.create_experiment(
            parameters=[design_matrix.parameter_configuration]
        )
        ensemble = storage.create_ensemble(
            experiment_id, name="default", ensemble_size=ensemble_size
        )
        if expect_error:
            with pytest.raises(KeyError):
                save_design_matrix_to_ensemble(
                    design_matrix.design_matrix_df, ensemble, reals
                )
        else:
            save_design_matrix_to_ensemble(
                design_matrix.design_matrix_df, ensemble, reals
            )
            params = ensemble.load_parameters(DESIGN_MATRIX_GROUP).drop("realization")
            assert isinstance(params, pl.DataFrame)
            assert params.columns == ["a", "b", "c"]
            np.testing.assert_array_almost_equal(params["a"].to_list(), a_values)
            np.testing.assert_array_almost_equal(params["b"].to_list(), b_values)
            np.testing.assert_array_almost_equal(params["c"].to_list(), c_values)


def test_load_gen_kw_not_sorted(storage, tmpdir, snapshot):
    """
    This test checks two things, loading multiple parameters and
    loading log parameters.
    """
    with tmpdir.as_cwd():
        config = dedent(
            """
        NUM_REALIZATIONS 10
        GEN_KW PARAM_2 template.txt kw.txt prior2.txt
        GEN_KW PARAM_1 template.txt kw.txt prior1.txt
        RANDOM_SEED 1234
        """
        )
        with open("config.ert", mode="w", encoding="utf-8") as fh:
            fh.writelines(config)
        with open("template.txt", mode="w", encoding="utf-8") as fh:
            fh.writelines("MY_KEYWORD <MY_KEYWORD>")
        with open("prior1.txt", mode="w", encoding="utf-8") as fh:
            fh.writelines("MY_KEYWORD1 LOGUNIF 0.1 1")
        with open("prior2.txt", mode="w", encoding="utf-8") as fh:
            fh.writelines("MY_KEYWORD2 LOGUNIF 0.1 1")

        ert_config = ErtConfig.from_file("config.ert")

        experiment_id = storage.create_experiment(
            parameters=ert_config.ensemble_config.parameter_configuration
        )
        ensemble_size = 10
        ensemble = storage.create_ensemble(
            experiment_id, name="default", ensemble_size=ensemble_size
        )

        sample_prior(ensemble, range(ensemble_size), random_seed=1234)

        data = ensemble.load_all_gen_kw_data()
        snapshot.assert_match(data.round(12).to_csv(), "gen_kw_unsorted")


def test_gen_kw_collector(snake_oil_default_storage, snapshot):
    data = snake_oil_default_storage.load_all_gen_kw_data()
    snapshot.assert_match(data.round(6).to_csv(), "gen_kw_collector.csv")

    with pytest.raises(KeyError):
        # realization 60:
        _ = data.loc[60]

    data = snake_oil_default_storage.load_all_gen_kw_data(
        "SNAKE_OIL_PARAM",
    )[["SNAKE_OIL_PARAM:OP1_PERSISTENCE", "SNAKE_OIL_PARAM:OP1_OFFSET"]]
    snapshot.assert_match(data.round(6).to_csv(), "gen_kw_collector_2.csv")

    with pytest.raises(KeyError):
        _ = data["SNAKE_OIL_PARAM:OP1_DIVERGENCE_SCALE"]

    realization_index = 3
    data = snake_oil_default_storage.load_all_gen_kw_data(
        "SNAKE_OIL_PARAM",
        realization_index=realization_index,
    )["SNAKE_OIL_PARAM:OP1_PERSISTENCE"]
    snapshot.assert_match(data.round(6).to_csv(), "gen_kw_collector_3.csv")

    non_existing_realization_index = 150
    with pytest.raises((IndexError, KeyError)):
        _ = snake_oil_default_storage.load_all_gen_kw_data(
            "SNAKE_OIL_PARAM",
            realization_index=non_existing_realization_index,
        )["SNAKE_OIL_PARAM:OP1_PERSISTENCE"]


def test_keyword_type_checks(snake_oil_default_storage):
    assert (
        "BPR:1,3,8"
        in snake_oil_default_storage.experiment.response_type_to_response_keys[
            "summary"
        ]
    )


def test_keyword_type_checks_missing_key(snake_oil_default_storage):
    assert (
        "nokey"
        not in snake_oil_default_storage.experiment.response_type_to_response_keys[
            "summary"
        ]
    )


@pytest.mark.filterwarnings("ignore:.*Use load_responses.*:DeprecationWarning")
def test_data_fetching_missing_key(snake_oil_case):
    with open_storage(snake_oil_case.ens_path, mode="w") as storage:
        experiment = storage.create_experiment()
        empty_case = experiment.create_ensemble(name="new_case", ensemble_size=25)

        data = [
            empty_case.load_all_gen_kw_data("nokey", None),
        ]

        for dataframe in data:
            assert isinstance(dataframe, DataFrame)
            assert dataframe.empty


@dataclass
class Ensemble:
    uuid: UUID
    parameter_values: dict[str, Any] = field(default_factory=dict)
    response_values: dict[str, Any] = field(default_factory=dict)
    failure_messages: dict[int, str] = field(default_factory=dict)


@dataclass
class Experiment:
    uuid: UUID
    ensembles: dict[UUID, Ensemble] = field(default_factory=dict)
    parameters: list[ParameterConfig] = field(default_factory=list)
    responses: list[ResponseConfig] = field(default_factory=list)
    observations: dict[str, pl.DataFrame] = field(default_factory=dict)


@settings(max_examples=250)
class StatefulStorageTest(RuleBasedStateMachine):
    """
    This test runs several commands against storage and
    checks its return values against a simple key-value store
    (the model).

    see https://hypothesis.readthe@docs.io/en/latest/stateful.html

    When the test fails, you get a printout like this:

    .. code-block:: text

        state = StatefulStorageTest()
        v1 = state.create_grid(egrid=EGrid(...))
        v2 = state.create_field_list(fields=[...])
        v3 = state.create_experiment(obs=EnkfObs(...), parameters=[...], responses=[...])
        v4 = state.create_ensemble(ensemble_size=1, model_experiment=v3)
        v5 = state.create_ensemble_from_prior(prior=v4)
        state.get_ensemble(model_ensemble=v5)
        state.teardown()

    This describes which rules are run (like create_experiment which corresponds to
    the same action storage api endpoint: self.storage.create_experiment), and which
    parameters are applied (e.g. v1 is in the grid bundle and is created by the rule
    state.create_grid).
    """  # noqa: E501

    def __init__(self) -> None:
        super().__init__()
        self.tmpdir = tempfile.mkdtemp(prefix="StatefulStorageTest")
        self.storage = open_storage(self.tmpdir + "/storage/", "w")
        note(f"storage path is: {self.storage.path}")
        self.model: dict[UUID, Experiment] = {}
        assert list(self.storage.ensembles) == []

        # Realization to save/delete params/responses
        # (all other reals are not modified throughout every run of this test)
        self.iens_to_edit = 0

    experiments = Bundle("experiments")
    ensembles = Bundle("ensembles")
    field_list = Bundle("field_list")
    grid = Bundle("grid")

    @initialize(target=grid, egrid=egrids)
    def create_grid(self, egrid):
        grid_file = self.tmpdir + "/grid.egrid"
        egrid.to_file(grid_file)
        return (grid_file, egrid)

    @initialize(
        target=field_list,
        fields=grid.flatmap(fields),
    )
    def create_field_list(self, fields):
        return fields

    @rule()
    def double_open_timeout(self):
        # Opening with write access will timeout when
        # already opened with mode="w" somewhere else
        with (
            patch("ert.storage.local_storage.LocalStorage.LOCK_TIMEOUT", 0.0),
            pytest.raises(ErtStorageException),
        ):
            open_storage(self.tmpdir + "/storage/", mode="w")

    @rule()
    def reopen(self):
        """
        closes as reopens the storage to ensure
        that doesn't effect its contents
        """
        ensembles = sorted(e.id for e in self.storage.ensembles)
        self.storage.close()
        self.storage = open_storage(self.tmpdir + "/storage/", mode="w")
        assert ensembles == sorted(e.id for e in self.storage.ensembles)

    @rule(
        target=experiments,
        parameters=st.one_of(parameter_configs, field_list),
        responses=response_configs,
        obs=observations,
    )
    def create_experiment(
        self,
        parameters: list[ParameterConfig],
        responses: list[ResponseConfig],
        obs: EnkfObs,
    ):
        experiment_id = self.storage.create_experiment(
            parameters=parameters, responses=responses, observations=obs.datasets
        ).id
        model_experiment = Experiment(experiment_id)
        model_experiment.parameters = parameters
        model_experiment.responses = responses
        model_experiment.observations = obs.datasets

        # Ensure that there is at least one ensemble in the experiment
        # to avoid https://github.com/equinor/ert/issues/7040
        ensemble = self.storage.create_ensemble(experiment_id, ensemble_size=1)
        model_experiment.ensembles[ensemble.id] = Ensemble(ensemble.id)

        self.model[model_experiment.uuid] = model_experiment

        return model_experiment

    @rule(
        model_ensemble=ensembles,
        field_data=grid.flatmap(lambda g: arrays(np.float32, shape=g[1].shape)),
    )
    def save_field(self, model_ensemble: Ensemble, field_data):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)

        # Ensembles w/ parent failure will never have parameters written to them
        assume(
            storage_ensemble.get_ensemble_state()[self.iens_to_edit]
            != RealizationStorageState.PARENT_FAILURE
        )

        parameters = model_ensemble.parameter_values.values()
        fields = [p for p in parameters if isinstance(p, Field)]
        for f in fields:
            model_ensemble.parameter_values[f.name] = field_data
            storage_ensemble.save_parameters(
                f.name,
                1,
                xr.DataArray(
                    field_data,
                    name="values",
                    dims=["x", "y", "z"],  # type: ignore
                ).to_dataset(),
            )

    @rule(
        model_ensemble=ensembles,
        field_data=grid.flatmap(lambda g: arrays(np.float32, shape=g[1].shape)),
    )
    def write_error_in_save_field(self, model_ensemble: Ensemble, field_data):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)

        # Ensembles w/ parent failure will never have parameters written to them
        assume(
            storage_ensemble.get_ensemble_state()[self.iens_to_edit]
            != RealizationStorageState.PARENT_FAILURE
        )

        parameters = model_ensemble.parameter_values.values()
        fields = [p for p in parameters if isinstance(p, Field)]

        assume(
            not storage_ensemble.get_realization_mask_with_parameters()[
                self.iens_to_edit
            ]
        )
        for f in fields:
            with (
                patch(
                    "ert.storage.local_storage.NamedTemporaryFile",
                    RaisingWriteNamedTemporaryFile,
                ) as temp_file,
                pytest.raises(RuntimeError),
            ):
                storage_ensemble.save_parameters(
                    f.name,
                    self.iens_to_edit,
                    xr.DataArray(
                        field_data,
                        name="values",
                        dims=["x", "y", "z"],  # type: ignore
                    ).to_dataset(),
                )

            assert temp_file.entered
        assert not storage_ensemble.get_realization_mask_with_parameters()[
            self.iens_to_edit
        ]

    @rule(
        model_ensemble=ensembles,
    )
    def get_parameters(self, model_ensemble: Ensemble):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)
        parameter_names = model_ensemble.parameter_values.keys()

        for f in parameter_names:
            parameter_data = storage_ensemble.load_parameters(f, self.iens_to_edit)
            xr.testing.assert_equal(
                model_ensemble.parameter_values[f],
                parameter_data["values"],
            )

    @rule(model_ensemble=ensembles, data=st.data())
    def save_summary(self, model_ensemble: Ensemble, data):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)
        storage_experiment = storage_ensemble.experiment

        assume(
            storage_ensemble.get_ensemble_state()[self.iens_to_edit]
            != RealizationStorageState.PARENT_FAILURE
        )

        # Enforce the summary data to respect the
        # scheme outlined in the response configs
        smry_config = storage_experiment.response_configuration.get("summary")

        if not smry_config:
            assume(False)
            raise AssertionError

        expected_summary_keys = (
            st.just(smry_config.keys)
            if smry_config.has_finalized_keys
            else st.lists(summary_variables(), min_size=1)
        )

        summaries_strategy = summaries(
            summary_keys=expected_summary_keys,
            start_date=st.datetimes(
                min_value=datetime.strptime("1969-1-1", "%Y-%m-%d"),
                max_value=datetime.strptime("2010-1-1", "%Y-%m-%d"),
            ),
            time_deltas=st.lists(
                st.floats(
                    min_value=0.1,
                    max_value=365,
                    allow_nan=False,
                    allow_infinity=False,
                ),
                min_size=2,
                max_size=10,
            ),
        )
        summary_data = data.draw(summaries_strategy)

        responses = storage_experiment.response_configuration.values()
        summary_configs = [p for p in responses if isinstance(p, SummaryConfig)]
        assume(summary_configs)
        summary = summary_configs[0]
        assume(summary.name not in model_ensemble.response_values)
        smspec, unsmry = summary_data
        smspec.to_file(self.tmpdir + f"/{summary.input_files[0]}.SMSPEC")
        unsmry.to_file(self.tmpdir + f"/{summary.input_files[0]}.UNSMRY")

        try:
            ds = summary.read_from_file(self.tmpdir, self.iens_to_edit, 0)
        except Exception as e:  # no match in keys
            assume(False)
            raise AssertionError from e
        storage_ensemble.save_response(summary.response_type, ds, self.iens_to_edit)

        model_ensemble.response_values[summary.name] = ds

        model_experiment = self.model[storage_experiment.id]
        response_keys = set(ds["response_key"].unique())

        model_smry_config = next(
            config for config in model_experiment.responses if config.name == "summary"
        )

        if not model_smry_config.has_finalized_keys:
            model_smry_config.keys = sorted(response_keys)
            model_smry_config.has_finalized_keys = True

    @rule(model_ensemble=ensembles)
    def get_responses(self, model_ensemble: Ensemble):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)
        response_types = model_ensemble.response_values.keys()

        for response_type in response_types:
            ensemble_data = storage_ensemble.load_responses(
                response_type, (self.iens_to_edit,)
            )
            model_data = model_ensemble.response_values[response_type]
            assert ensemble_data.equals(model_data)

    @rule(model_ensemble=ensembles, parameter=words)
    def load_unknown_parameter(self, model_ensemble: Ensemble, parameter: str):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)
        experiment_id = storage_ensemble.experiment_id
        parameter_names = [p.name for p in self.model[experiment_id].parameters]
        assume(parameter not in parameter_names)
        with pytest.raises(
            KeyError, match=f"{parameter} is not registered to the experiment"
        ):
            _ = storage_ensemble.load_parameters(parameter, self.iens_to_edit)

    @rule(
        target=ensembles,
        model_experiment=experiments,
        ensemble_size=ensemble_sizes,
    )
    def create_ensemble(self, model_experiment: Experiment, ensemble_size: int):
        ensemble = self.storage.create_ensemble(
            model_experiment.uuid, ensemble_size=ensemble_size
        )
        assert ensemble in self.storage.ensembles
        model_ensemble = Ensemble(ensemble.id)
        model_experiment.ensembles[ensemble.id] = model_ensemble

        is_expecting_responses = any(
            len(config.keys) for config in model_experiment.responses
        )

        if is_expecting_responses:
            assert all(
                (RealizationStorageState.UNDEFINED in s)
                for s in ensemble.get_ensemble_state()
            )
            assert np.all(
                np.logical_not(ensemble.get_realization_mask_with_responses())
            )
        else:
            assert all(
                RealizationStorageState.RESPONSES_LOADED in state
                for state in ensemble.get_ensemble_state()
            )
            assert np.all(ensemble.get_realization_mask_with_responses())

        return model_ensemble

    @rule(
        target=ensembles,
        prior=ensembles,
    )
    def create_ensemble_from_prior(self, prior: Ensemble):
        prior_ensemble = self.storage.get_ensemble(prior.uuid)
        experiment_id = prior_ensemble.experiment_id
        size = prior_ensemble.ensemble_size
        ensemble = self.storage.create_ensemble(
            experiment_id, ensemble_size=size, prior_ensemble=prior.uuid
        )
        assert ensemble in self.storage.ensembles
        model_ensemble = Ensemble(ensemble.id)
        model_experiment = self.model[experiment_id]
        model_experiment.ensembles[ensemble.id] = model_ensemble

        prior_state = prior_ensemble.get_ensemble_state()
        edited_prior_state = prior_state[self.iens_to_edit]

        posterior_state = ensemble.get_ensemble_state()
        edited_posterior_state = posterior_state[self.iens_to_edit]

        if edited_prior_state.intersection(
            {
                RealizationStorageState.UNDEFINED,
                RealizationStorageState.PARENT_FAILURE,
                RealizationStorageState.LOAD_FAILURE,
            }
        ):
            assert RealizationStorageState.PARENT_FAILURE in edited_posterior_state
        else:
            is_expecting_responses = (
                sum(len(config.keys) for config in model_experiment.responses) > 0
            )
            # If expecting no responses, i.e., it has empty .keys in all response
            # configs, it will be a HAS_DATA even if no responses were ever saved
            if not is_expecting_responses:
                assert (
                    RealizationStorageState.RESPONSES_LOADED in edited_posterior_state
                )
            else:
                assert self.iens_to_edit not in prior.failure_messages
                assert RealizationStorageState.UNDEFINED in edited_posterior_state

        return model_ensemble

    @rule(model_experiment=experiments)
    def get_experiment(self, model_experiment: Experiment):
        storage_experiment = self.storage.get_experiment(model_experiment.uuid)
        assert storage_experiment.id == model_experiment.uuid
        assert sorted(model_experiment.ensembles) == sorted(
            e.id for e in storage_experiment.ensembles
        )
        assert (
            list(storage_experiment.response_configuration.values())
            == model_experiment.responses
        )
        for obskey, obs in model_experiment.observations.items():
            assert obskey in storage_experiment.observations
            assert obs.equals(storage_experiment.observations[obskey])

    @rule(model_ensemble=ensembles)
    def get_ensemble(self, model_ensemble: Ensemble):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)
        assert storage_ensemble.id == model_ensemble.uuid

    @rule(model_ensemble=ensembles, data=st.data(), message=st.text())
    def set_failure(self, model_ensemble: Ensemble, data: st.DataObject, message: str):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)
        assert storage_ensemble.id == model_ensemble.uuid

        realization = data.draw(
            st.integers(min_value=0, max_value=storage_ensemble.ensemble_size - 1)
        )

        storage_ensemble.set_failure(
            realization, RealizationStorageState.PARENT_FAILURE, message
        )
        model_ensemble.failure_messages[realization] = message

    @rule(model_ensemble=ensembles, data=st.data(), message=st.text())
    def write_error_in_set_failure(
        self,
        model_ensemble: Ensemble,
        data: st.DataObject,
        message: str,
    ):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)
        realization = data.draw(
            st.integers(min_value=0, max_value=storage_ensemble.ensemble_size - 1)
        )
        assume(not storage_ensemble.has_failure(realization))

        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)

        with (
            patch(
                "ert.storage.local_storage.NamedTemporaryFile",
                RaisingWriteNamedTemporaryFile,
            ) as f,
            pytest.raises(RuntimeError),
        ):
            storage_ensemble.set_failure(
                realization, RealizationStorageState.PARENT_FAILURE, message
            )
        assert f.entered

        assert not storage_ensemble.has_failure(realization)

    @rule(model_ensemble=ensembles, data=st.data())
    def get_failure(self, model_ensemble: Ensemble, data: st.DataObject):
        storage_ensemble = self.storage.get_ensemble(model_ensemble.uuid)
        realization = data.draw(
            st.integers(min_value=0, max_value=storage_ensemble.ensemble_size - 1)
        )
        fail = self.storage.get_ensemble(model_ensemble.uuid).get_failure(realization)
        if realization in model_ensemble.failure_messages:
            assert fail is not None
            assert fail.message == model_ensemble.failure_messages[realization]
        else:
            assert fail is None or "Failure from prior" in fail.message

    def teardown(self):
        if self.storage is not None:
            self.storage.close()
        if self.tmpdir is not None:
            shutil.rmtree(self.tmpdir)


TestStorage = pytest.mark.integration_test(StatefulStorageTest.TestCase)
