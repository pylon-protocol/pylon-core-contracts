PWD := $(shell pwd)
BASENAME := $(shell basename $(PWD))

all: build-dev build-prod

check:
	@mkdir -p ./artifacts
	@cargo check

build-dev: check
	@cargo wasm
	@ls ./target/wasm32-unknown-unknown/release/*.wasm | xargs -I{} cp {} ./artifacts

build-prod: check
	docker run --rm -v "$(PWD)":/code \
  	  --mount type=volume,source="$(BASENAME)_cache",target=/code/target \
  	  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  	  cosmwasm/workspace-optimizer:0.12.1
