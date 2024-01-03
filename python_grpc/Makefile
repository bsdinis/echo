SOURCES=$(wildcard *.py)

.PHONY: all
all: fmt mypy

.PHONY: fmt
fmt: $(SOURCES)
	autopep8 -i -aa $(SOURCES)

.PHONY: mypy
mypy: $(SOURCES)
	mypy --ignore-missing-imports .

grpc: echo.proto
	python -m grpc_tools.protoc -I. --python_out=. --pyi_out=. --grpc_python_out=. echo.proto
