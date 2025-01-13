#!/usr/bin/env python3

import random

from qtpy.QtCore import QAbstractItemModel, QAbstractProxyModel, QModelIndex, Qt
from qtpy.QtWidgets import QApplication, QListView, QTableView

main_input = [
    ("Belgium", ["Brussels", "Antwerp", "Ghent", "Liège", "Bruges"]),
    ("Sweden", ["Stockholm", "Gothenburg", "Malmö", "Uppsala", "Västerås"]),
    ("France", ["Paris", "Marseille", "Lyon", "Toulouse", "Nice"]),
    ("Germany", ["Berlin", "Hamburg", "Munich", "Cologne", "Frankfurt"]),
    ("Italy", ["Rome", "Milan", "Naples", "Turin", "Palermo"]),
    ("Spain", ["Madrid", "Barcelona", "Valencia", "Seville", "Zaragoza"]),
    ("Netherlands", ["Amsterdam", "Rotterdam", "The Hague", "Utrecht", "Eindhoven"]),
    ("Poland", ["Warsaw", "Kraków", "Łódź", "Wrocław", "Poznań"]),
    ("United Kingdom", ["London", "Birmingham", "Manchester", "Glasgow", "Liverpool"]),
    ("Greece", ["Athens", "Thessaloniki", "Patras", "Heraklion", "Larissa"]),
]


class City:
    def __init__(self, name):
        self.name = name
        self.population = random.randint(1_000, 6_000_000)
        self.color = random.choice(
            [
                "red",
                "blue",
                "green",
                "yellow",
                "black",
                "white",
                "purple",
                "orange",
                "pink",
                "brown",
            ]
        )


CITY_COLUMNS = ["Name", "Population", "Color"]


class Country:
    def __init__(self, name, cities):
        self.name = name
        self.population = random.randint(20_000, 50_000_000)
        self.area = random.randint(20, 1000)
        self.cities = [City(city) for city in cities]


COUNTRY_COLUMNS = ["Name", "Population", "Area"]

generated_countries = [Country(name, cities) for name, cities in main_input]


class MyModel(QAbstractItemModel):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._countries = generated_countries

    def data(self, index, role):
        if role == Qt.DisplayRole:
            if not index.parent().isValid():
                country = self._countries[index.row()]
                if index.column() == 0:
                    return country.name
                elif index.column() == 1:
                    return country.population
                elif index.column() == 2:
                    return country.area
            else:
                city = index.internalPointer()
                return city.name

    def headerData(self, section, orientation, role):
        if role == Qt.DisplayRole:
            if orientation == Qt.Horizontal:
                return COUNTRY_COLUMNS[section]
            else:
                return section

    def index(self, row, column, parent):
        if not parent.isValid():
            return self.createIndex(row, column, self._countries[row])
        else:
            country = parent.internalPointer()
            return self.createIndex(row, column, country.cities[row])

    def parent(self, index):
        if isinstance(index.internalPointer(), Country):
            return QModelIndex()
        else:
            city = index.internalPointer()
            for country in self._countries:
                if city in country.cities:
                    return self.createIndex(self._countries.index(country), 0, country)
        return QModelIndex()

    def rowCount(self, parent):
        if not parent.isValid():
            return len(self._countries)
        else:
            country = parent.internalPointer()
            return len(country.cities)

    def columnCount(self, parent):
        if not parent.isValid():
            return len(COUNTRY_COLUMNS)
        else:
            return 1


class CitiesProxyModel(QAbstractProxyModel):
    def __init__(self, source_model, selection_model):
        super().__init__()
        self.setSourceModel(source_model)
        self._selection_model = selection_model

    def rowCount(self, parent=QModelIndex()):
        selected_indexes = self._selection_model.selectedIndexes()
        if selected_indexes:
            selected_row = selected_indexes[0].row()
            country = self.sourceModel()._countries[selected_row]
            return len(country.cities)
        return 0

    def columnCount(self, parent=QModelIndex()):
        return 1

    def index(self, row, column, parent=QModelIndex()):
        if self.rowCount() > 0:
            return self.createIndex(row, column)
        return QModelIndex()

    def parent(self, index):
        return QModelIndex()

    def mapFromSource(self, source_index):
        selected_indexes = self._selection_model.selectedIndexes()
        if selected_indexes:
            selected_row = selected_indexes[0].row()
            country = self.sourceModel()._countries[selected_row]
            if source_index.internalPointer() in country.cities:
                row = country.cities.index(source_index.internalPointer())
                return self.createIndex(row, source_index.column())
        return QModelIndex()

    def mapToSource(self, proxy_index):
        selected_indexes = self._selection_model.selectedIndexes()
        if selected_indexes:
            selected_row = selected_indexes[0].row()
            country = self.sourceModel()._countries[selected_row]
            if proxy_index.row() < len(country.cities):
                parent_index = self.sourceModel().index(selected_row, 0, QModelIndex())
                return self.sourceModel().index(
                    proxy_index.row(), proxy_index.column(), parent_index
                )
        return QModelIndex()


def main():
    app = QApplication([])
    main_model = MyModel()

    country_table = QTableView()
    country_table.setSelectionBehavior(QTableView.SelectRows)
    country_table.setWindowTitle("Countries")
    country_table.setModel(main_model)
    country_table.show()

    city_list = QListView()
    city_proxy_model = CitiesProxyModel(main_model, country_table.selectionModel())
    city_list.setWindowTitle("Cities")
    city_list.setModel(city_proxy_model)
    city_list.show()

    country_table.selectionModel().selectionChanged.connect(
        city_proxy_model.layoutChanged.emit
    )

    app.exec_()


main()
