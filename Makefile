HYPERFINE := $(shell command -v hyperfine 2> /dev/null)
IMG ?= policy-server:latest

.PHONY: build
build:
	cargo build --release

.PHONY: fmt
fmt:
	cargo fmt --all -- --check

.PHONY: lint
lint:
	cargo clippy -- -D warnings

.PHONY: test
test: fmt lint
	cargo test --workspace

.PHONY: clean
clean:
	cargo clean

.PHONY: tag
tag:
	@git tag "${TAG}" || (echo "Tag ${TAG} already exists. If you want to retag, delete it manually and re-run this command" && exit 1)
	@git-chglog --output CHANGELOG.md
	@git commit -m 'Update CHANGELOG.md' -- CHANGELOG.md
	@git tag -f "${TAG}"

.PHONY: docker-build
docker-build: test ## Build docker image with the manager.
	docker build -t ${IMG} .

sbom-tool:
	curl -Lo sbom-tool https://github.com/microsoft/sbom-tool/releases/download/v0.1.13/sbom-tool-linux-x64
	chmod +x sbom-tool

.PHONY: sbom
sbom: sbom-tool
	./sbom-tool generate -b ./target/release -D true -bc . -pn policy-server -pv 1.0.0 -nsb https://kubewarden.io -nsu policy-server -V Verbose
