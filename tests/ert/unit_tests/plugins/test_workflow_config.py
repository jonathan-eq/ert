import logging
from unittest.mock import Mock

import pytest

import ert
from ert.plugins import workflow_config


def test_workflow_config_duplicate_log_message(caplog, monkeypatch):
    def get_mock_config():
        workflow_mock = Mock(spec=workflow_config.ErtScriptWorkflow)
        workflow_mock.name = "same_name"
        return workflow_mock

    config = workflow_config.WorkflowConfigs()

    # Create duplicate workflows
    workflows = [get_mock_config(), get_mock_config()]

    monkeypatch.setattr(config, "_workflows", workflows)
    with caplog.at_level(logging.INFO):
        config.get_workflows()
    assert "Duplicate workflow name: same_name, skipping" in caplog.text


@pytest.mark.parametrize(
    "name, expected", [(None, "default_name"), ("some_name", "some_name")]
)
def test_workflow_config_init_name(monkeypatch, name, expected):
    mock_func = Mock
    mock_func.__name__ = "default_name"

    monkeypatch.setattr(ert.config.workflow_job.WorkflowJob, "__post_init__", Mock())
    workflow = workflow_config.ErtScriptWorkflow(mock_func, name=name)

    assert workflow.name == expected
    assert workflow.ert_script == mock_func
