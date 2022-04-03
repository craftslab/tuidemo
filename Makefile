# Build

.PHONY: FORCE

build: rs-build
.PHONY: build

clean: rs-clean
.PHONY: clean

docker: rs-docker
.PHONY: docker

install: rs-install
.PHONY: install

lint: rs-lint
.PHONY: lint

test: rs-test
.PHONY: test


# Non-PHONY targets (real files)

rs-build: FORCE
	./script/build.sh

rs-clean: FORCE
	./script/clean.sh

rs-docker: FORCE
	./script/docker.sh

rs-install: FORCE
	./script/install.sh

rs-lint: FORCE
	./script/lint.sh

rs-test: FORCE
	./script/test.sh
