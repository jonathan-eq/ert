import numbers
from copy import deepcopy

import pytest
from pydantic import ValidationError

from ert.config import ConfigWarning
from everest.config import EverestConfig, InputConstraintConfig
from everest.config.control_config import ControlConfig
from everest.config.control_variable_config import (
    ControlVariableConfig,
    ControlVariableGuessListConfig,
)
from everest.config.well_config import WellConfig
from everest.optimizer.everest2ropt import everest2ropt
from tests.everest.utils import relpath

cfg_dir = relpath("test_data", "mocked_test_case")
mocked_config = relpath(cfg_dir, "mocked_test_case.yml")


def test_controls_initialization():
    exp_grp_name = "group"

    config = EverestConfig.load_file(mocked_config)
    assert config.controls is not None
    group = config.controls[0]
    assert group.variables is not None

    assert exp_grp_name == group.name

    for c in group.variables:
        assert isinstance(c.name, str)
        assert isinstance(c.initial_guess, numbers.Number)

    a_ctrl_name = group.variables[0].name
    config.controls.append(
        ControlConfig(
            name=exp_grp_name,
            type="well_control",
            variables=[
                ControlVariableConfig(
                    name=a_ctrl_name,
                    min=0,
                    max=1,
                    initial_guess=0.5,
                )
            ],
        )
    )
    with pytest.raises(
        ValidationError,
        match=r"Subfield\(s\) `name` must be unique",
    ):
        EverestConfig.model_validate(config.to_dict())

    config.controls[1].name = exp_grp_name + "_new"
    EverestConfig.model_validate(config.to_dict())


def test_control_variable_duplicate_name_no_index():
    with pytest.raises(
        ValidationError, match=r"Subfield\(s\) `name.index` must be unique"
    ):
        ControlConfig(
            name="group",
            type="generic_control",
            initial_guess=0.5,
            variables=[
                ControlVariableConfig(name="w00", min=0, max=1),
                ControlVariableConfig(
                    name="w00", min=0, max=1
                ),  # This is the duplicate
            ],
        )


def test_control_variable_index_inconsistency():
    with pytest.raises(
        ValidationError, match="for all of the variables or for none of them"
    ):
        ControlConfig(
            name="group",
            type="generic_control",
            initial_guess=0.5,
            variables=[
                ControlVariableConfig(name="w00", min=0, max=1),
                ControlVariableConfig(name="w01", min=0, max=1, index=0),
            ],
        )


def test_control_variable_duplicate_name_and_index():
    with pytest.raises(
        ValidationError, match=r"Subfield\(s\) `name.index` must be unique"
    ):
        ControlConfig(
            name="group",
            type="generic_control",
            initial_guess=0.5,
            variables=[
                ControlVariableConfig(name="w00", min=0, max=1, index=0),
                ControlVariableConfig(name="w00", min=0, max=1, index=0),
            ],
        )


def test_input_constraint_name_mismatch_with_indexed_variables():
    with pytest.raises(
        ValidationError,
        match="does not match any instance of "
        "control_name\\.variable_name\\.variable_index",
    ):
        EverestConfig.with_defaults(
            controls=[
                ControlConfig(
                    name="group",
                    type="generic_control",
                    initial_guess=0.5,
                    variables=[
                        ControlVariableConfig(name="w00", min=0, max=1, index=0),
                        ControlVariableConfig(name="w01", min=0, max=1, index=0),
                    ],
                )
            ],
            input_constraints=[
                InputConstraintConfig(
                    upper_bound=1,
                    lower_bound=0,
                    weights={"group.w00": 0.1},
                )
            ],
        )


def test_input_constraint_deprecated_indexed_name_format_warns():
    with pytest.warns(
        ConfigWarning, match="Deprecated input control name: group.w00-0"
    ):
        EverestConfig.with_defaults(
            controls=[
                ControlConfig(
                    name="group",
                    type="generic_control",
                    initial_guess=0.5,
                    variables=[
                        ControlVariableConfig(name="w00", min=0, max=1, index=0),
                        ControlVariableConfig(name="w01", min=0, max=1, index=0),
                    ],
                )
            ],
            input_constraints=[
                InputConstraintConfig(
                    upper_bound=1,
                    lower_bound=0,
                    weights={
                        "group.w00-0": 0.1
                    },  # This specific format is deprecated [7].
                )
            ],
        )


def test_control_variable_initial_guess_below_min():
    with pytest.raises(ValidationError, match="initial_guess"):
        ControlConfig(
            name="control",
            type="well_control",
            variables=[
                ControlVariableConfig(name="w00", min=0.5, max=1.0, initial_guess=0.3)
            ],
        )


def test_control_variable_initial_guess_above_max():
    with pytest.raises(ValidationError, match="initial_guess"):
        ControlConfig(
            name="control",
            type="well_control",
            variables=[
                ControlVariableConfig(name="w00", min=0.5, max=1.0, initial_guess=1.3)
            ],
        )


def test_control_variable_name():
    """We would potentially like to support variable names with
    underscores, but currently Seba is using this as a separator between
    the group name and the variable name in such a way that having an
    underscore in a variable name will not behave nicely..
    """
    config = EverestConfig.load_file(mocked_config)
    EverestConfig.model_validate(config.to_dict())

    illegal_name = "illegal.name.due.to.dots"
    config.controls[0].variables[0].name = illegal_name
    with pytest.raises(
        ValidationError,
        match="Variable name can not contain any dots",
    ):
        EverestConfig.model_validate(config.to_dict())

    weirdo_name = "something/with-symbols_=/()*&%$#!"
    new_config = EverestConfig.load_file(mocked_config)
    new_config.wells.append(WellConfig(name=weirdo_name))
    new_config.controls[0].variables[0].name = weirdo_name
    EverestConfig.model_validate(new_config.to_dict())


def test_control_none_well_variable_name():
    config = EverestConfig.load_file(mocked_config)
    EverestConfig.model_validate(config.to_dict())

    illegal_name = "nowell4sure"
    config.controls[0].variables[0].name = illegal_name
    with pytest.raises(
        ValidationError,
        match="Variable name does not match any well name",
    ):
        EverestConfig.model_validate(config.to_dict())


def test_control_variable_types(control_config: ControlConfig):
    if isinstance(control_config.variables[0], ControlVariableConfig):
        assert all(
            isinstance(variable, ControlVariableConfig)
            for variable in control_config.variables
        )
    else:
        assert all(
            isinstance(variable, ControlVariableGuessListConfig)
            for variable in control_config.variables
        )


@pytest.mark.parametrize(
    "variables",
    (
        pytest.param(
            [
                {"name": "w00", "initial_guess": 0.0626, "index": 0},
                {"name": "w00", "initial_guess": [0.063, 0.0617, 0.0621]},
            ],
            id="same name",
        ),
        pytest.param(
            [
                {"name": "w00", "initial_guess": 0.0626, "index": 0},
                {"name": "w01", "initial_guess": [0.0627, 0.0631, 0.0618, 0.0622]},
            ],
            id="different name",
        ),
    ),
)
def test_control_bad_variables(variables, control_data_no_variables: dict):
    data = deepcopy(control_data_no_variables)
    data["variables"] = variables
    with pytest.raises(ValidationError, match="3 validation errors"):
        ControlConfig.model_validate(data)


def test_controls_ordering_is_consistent_for_ropt_and_extparam():
    index_wise = ControlConfig(
        name="well_priorities",
        type="well_control",
        variables=[
            {"name": "WELL-1", "initial_guess": [0.58, 0.54, 0.5, 0.52]},
            {"name": "WELL-2", "initial_guess": [0.5, 0.58, 0.56, 0.54]},
            {"name": "WELL-3", "initial_guess": [0.56, 0.52, 0.58, 0.5]},
            {"name": "WELL-4", "initial_guess": [0.54, 0.56, 0.54, 0.58]},
            {"name": "WELL-5", "initial_guess": [0.52, 0.5, 0.52, 0.56]},
        ],
        control_type="real",
        min=0.0,
        max=1.0,
        perturbation_type="absolute",
        perturbation_magnitude=0.05,
        scaled_range=[0.0, 1.0],
    )

    var_wise = ControlConfig(
        name="well_priorities",
        type="well_control",
        variables=[
            {"name": "WELL-1", "initial_guess": 0.58, "index": 1},
            {"name": "WELL-1", "initial_guess": 0.54, "index": 2},
            {"name": "WELL-1", "initial_guess": 0.5, "index": 3},
            {"name": "WELL-1", "initial_guess": 0.52, "index": 4},
            {"name": "WELL-2", "initial_guess": 0.5, "index": 1},
            {"name": "WELL-2", "initial_guess": 0.58, "index": 2},
            {"name": "WELL-2", "initial_guess": 0.56, "index": 3},
            {"name": "WELL-2", "initial_guess": 0.54, "index": 4},
            {"name": "WELL-3", "initial_guess": 0.56, "index": 1},
            {"name": "WELL-3", "initial_guess": 0.52, "index": 2},
            {"name": "WELL-3", "initial_guess": 0.58, "index": 3},
            {"name": "WELL-3", "initial_guess": 0.5, "index": 4},
            {"name": "WELL-4", "initial_guess": 0.54, "index": 1},
            {"name": "WELL-4", "initial_guess": 0.56, "index": 2},
            {"name": "WELL-4", "initial_guess": 0.54, "index": 3},
            {"name": "WELL-4", "initial_guess": 0.58, "index": 4},
            {"name": "WELL-5", "initial_guess": 0.52, "index": 1},
            {"name": "WELL-5", "initial_guess": 0.5, "index": 2},
            {"name": "WELL-5", "initial_guess": 0.52, "index": 3},
            {"name": "WELL-5", "initial_guess": 0.56, "index": 4},
        ],
        control_type="real",
        min=0.0,
        max=1.0,
        perturbation_type="absolute",
        perturbation_magnitude=0.05,
        scaled_range=(0.0, 1.0),
    )

    ever_config_var_wise = EverestConfig.with_defaults(controls=[var_wise])
    ever_config_index_wise = EverestConfig.with_defaults(controls=[index_wise])

    ropt_var_wise = everest2ropt(
        ever_config_var_wise.controls,
        ever_config_var_wise.objective_functions,
        ever_config_var_wise.input_constraints,
        ever_config_var_wise.output_constraints,
        ever_config_var_wise.optimization,
        ever_config_var_wise.model,
        1234,
        "dummy",
    )

    ropt_index_wise = everest2ropt(
        ever_config_index_wise.controls,
        ever_config_index_wise.objective_functions,
        ever_config_index_wise.input_constraints,
        ever_config_index_wise.output_constraints,
        ever_config_index_wise.optimization,
        ever_config_index_wise.model,
        1234,
        "dummy",
    )

    assert (
        ropt_var_wise[0]["names"]["variable"] == ropt_index_wise[0]["names"]["variable"]
    )

    assert (
        ropt_var_wise[0]["names"]["variable"]
        == index_wise.to_ert_parameter_config().input_keys
    )

    assert (
        index_wise.to_ert_parameter_config().input_keys
        == var_wise.to_ert_parameter_config().input_keys
    )


def test_controls_ordering_disregards_index():
    var_wise = ControlConfig(
        name="well_priorities",
        type="well_control",
        variables=[
            {"name": "WELL-1", "initial_guess": 0.54, "index": 2},
            {"name": "WELL-1", "initial_guess": 0.58, "index": 1},
            {"name": "WELL-1", "initial_guess": 0.5, "index": 3},
            {"name": "WELL-2", "initial_guess": 0.58, "index": 2},
            {"name": "WELL-2", "initial_guess": 0.5, "index": 1},
            {"name": "WELL-2", "initial_guess": 0.56, "index": 3},
            {"name": "WELL-3", "initial_guess": 0.52, "index": 2},
            {"name": "WELL-3", "initial_guess": 0.56, "index": 1},
            {"name": "WELL-3", "initial_guess": 0.58, "index": 3},
        ],
        control_type="real",
        min=0.0,
        max=1.0,
        perturbation_type="absolute",
        perturbation_magnitude=0.05,
        scaled_range=(0.0, 1.0),
    )

    ever_config_var_wise = EverestConfig.with_defaults(controls=[var_wise])

    ropt_var_wise = everest2ropt(
        ever_config_var_wise.controls,
        ever_config_var_wise.objective_functions,
        ever_config_var_wise.input_constraints,
        ever_config_var_wise.output_constraints,
        ever_config_var_wise.optimization,
        ever_config_var_wise.model,
        1234,
        "dummy",
    )

    expected = [
        "well_priorities.WELL-1.2",
        "well_priorities.WELL-1.1",
        "well_priorities.WELL-1.3",
        "well_priorities.WELL-2.2",
        "well_priorities.WELL-2.1",
        "well_priorities.WELL-2.3",
        "well_priorities.WELL-3.2",
        "well_priorities.WELL-3.1",
        "well_priorities.WELL-3.3",
    ]
    assert (ropt_var_wise[0]["names"]["variable"]) == expected

    assert var_wise.to_ert_parameter_config().input_keys == expected
