import warnings
from collections.abc import Callable, Generator
from contextlib import contextmanager
from typing import Any, TextIO

from ert.ensemble_evaluator.event import WarningEvent


@contextmanager
def capture_specific_warning(
    warning_class_to_capture: type[Warning], propagate_warning: Callable[[Any], None]
) -> Generator[None, None, None]:
    original_warning_handler = warnings.showwarning

    def _custom_warning_handler(
        message: Warning | str,
        category: type[Warning],
        filename: str,
        lineno: int,
        file: TextIO | None = None,
        line: str | None = None,
    ) -> None:
        if issubclass(category, warning_class_to_capture):
            propagate_warning(WarningEvent(msg=str(message)))
        else:
            original_warning_handler(message, category, filename, lineno, file, line)

    warnings.showwarning = _custom_warning_handler
    try:
        yield
    finally:
        warnings.showwarning = original_warning_handler
