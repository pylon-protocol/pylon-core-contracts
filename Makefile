PWD := $(shell pwd)
BASENAME := $(shell basename $(PWD))

all: build deploy

build:
	docker run --rm -v "$(PWD)":/code \
	  --mount type=volume,source="$(BASENAME)_cache",target=/code/target \
	  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	  cosmwasm/workspace-optimizer:0.10.3

deploy:
ifndef network
	yarn --cwd ./deployer start
else
	yarn --cwd ./deployer start --network $(network)
endif

deploy-columbus:
	deploy network=columbus

deploy-tequila:
	deploy network=tequila