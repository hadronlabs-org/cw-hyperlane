APP_DIR ?= ../..
COMPOSE ?= docker-compose

build-axelar:
		@docker buildx build --load --build-context app=. -f dockerbuilds/Dockerfile.axelar -t axelar/core .

build-neutron:
		cd $(APP_DIR)/neutron && $(MAKE) build-docker-image

build-hermes:
		@docker build -f dockerbuilds/Dockerfile.hermes -t hermes-airdroptest .

build-all: build-neutron build-hermes build-axelar
