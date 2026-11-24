# RustOS Master Makefile

PYTHON ?= python3
DELIVERABLES = deliverables
ISO = $(DELIVERABLES)/rustos.iso

.PHONY: all iso run clean docker-build docker-run test

all: iso

iso:
	$(PYTHON) tools/build.py --iso

run: iso
	powershell -ExecutionPolicy Bypass -File launch_vbox.ps1

clean:
	$(PYTHON) tools/build.py --clean

docker-build:
	docker build -t rustos-builder .

docker-run:
	docker run --rm -v $(CURDIR)/deliverables:/rustos/deliverables rustos-builder

test:
	$(PYTHON) tools/tests/test_kernel.py
