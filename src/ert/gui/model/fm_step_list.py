from typing import Any, overload

from qtpy.QtCore import (
    QAbstractProxyModel,
    QModelIndex,
    QObject,
    Qt,
    QVariant,
    Slot,
)
from typing_extensions import override

from ert.ensemble_evaluator import identifiers as ids
from ert.gui.model.snapshot import (
    FM_STEP_COLUMN_SIZE,
    FM_STEP_COLUMNS,
    IsEnsembleRole,
    IsFMStepRole,
    IsRealizationRole,
    NodeRole,
    SnapshotModel,
)


class FMStepListProxyModel(QAbstractProxyModel):
    """This proxy model presents two-dimensional views (row-column) of
    forward model data for a specific realization in a specific iteration."""

    def __init__(self, parent: QObject | None, iter_: int, real_: int) -> None:
        super().__init__(parent=parent)
        self._iter = iter_
        self._real = real_

    @Slot(int, int)
    def set_real(self, iter_: int, real: int) -> None:
        """Called when the user clicks a specific realization in the run_dialog window."""
        self._iter = iter_
        self._real = real

    def _get_source_parent_index(self) -> QModelIndex:
        start = self.index(0, 0, QModelIndex())
        if not start.isValid():
            return QModelIndex()
        if start.internalPointer() is None:
            return QModelIndex()
        source_parent = self.mapToSource(start).parent()
        return source_parent

    @override
    def headerData(
        self,
        section: int,
        orientation: Qt.Orientation,
        role: int = Qt.ItemDataRole.DisplayRole,
    ) -> Any:
        if role == Qt.ItemDataRole.DisplayRole:
            if orientation == Qt.Orientation.Horizontal:
                header = FM_STEP_COLUMNS[section]
                if header in {ids.STDOUT, ids.STDERR}:
                    return header.upper()
                elif header in {ids.MAX_MEMORY_USAGE}:
                    header = header.replace("_", " ")
                return header.capitalize()
            if orientation == Qt.Orientation.Vertical:
                return section
        return QVariant()

    @override
    def columnCount(self, parent: QModelIndex | None = None) -> int:
        return FM_STEP_COLUMN_SIZE

    def rowCount(self, parent: QModelIndex | None = None) -> int:
        parent = parent if parent else QModelIndex()
        # JONAK - SHOULDNT PARENT ALWAYS BE VALID? AN FM STEP SHOULD ALWAYS HAVE A REALIZATION
        if not parent.isValid():
            source_model: SnapshotModel = self.sourceModel()
            assert source_model is not None
            source_index = self._get_source_parent_index()  # This should be a realindex
            print()
            if source_index.isValid():
                return source_model.rowCount(source_index)
        return 0

    @overload
    def parent(self, child: QModelIndex) -> QModelIndex: ...
    @overload
    def parent(self) -> QObject | None: ...
    @override
    def parent(self, child: QModelIndex | None = None) -> QObject | None:
        return QModelIndex()

    @override
    def index(
        self, row: int, column: int, parent: QModelIndex | None = None
    ) -> QModelIndex:
        parent = parent if parent else QModelIndex()
        if not parent.isValid():
            job_index = self.mapToSource(self.createIndex(row, column, parent))
            return self.createIndex(row, column, job_index.data(NodeRole))
        return QModelIndex()

    def mapToSource(self, proxyIndex: QModelIndex) -> QModelIndex:
        if proxyIndex.isValid():
            sm: SnapshotModel = self.sourceModel()
            assert sm is not None
            iter_index = sm.index(self._iter, 0, QModelIndex())
            if iter_index.isValid() and sm.hasChildren(iter_index):
                real_index = sm.index(self._real, 0, iter_index)
                if real_index.isValid() and sm.hasChildren(real_index):
                    return sm.index(proxyIndex.row(), proxyIndex.column(), real_index)
        return QModelIndex()

    def mapFromSource(self, src_index: QModelIndex) -> QModelIndex:
        return (
            self.index(src_index.row(), src_index.column(), QModelIndex())
            if src_index.isValid() and self._accept_index(src_index)
            else QModelIndex()
        )

    def _accept_index(self, index: QModelIndex) -> bool:
        if not index.internalPointer() or not index.data(IsFMStepRole):
            return False

        # traverse upwards and check real and iter against parents of this index
        while index.isValid() and index.internalPointer():
            if (index.data(IsRealizationRole) and (index.row() != self._real)) or (
                index.data(IsEnsembleRole) and (index.row() != self._iter)
            ):
                return False
            index = index.parent()
        return True

    def get_iter(self) -> int:
        return self._iter

    def get_real(self) -> int:
        return
        return self._real
