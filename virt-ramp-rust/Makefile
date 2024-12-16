.PHONY: all build virt sign

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_\-.*]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

all: build virt sign ## Build and sign

build: ## Build the project
	wash build --build-only -p ./wasmcloud.build.toml

virt: ## Build the project and create a Wasi-Virt module
	wasi-virt ./build/p1/http_static_assets.wasm --mount /=./static -o ./build/http_static_assets.wasm --allow-fs=false
	# cp ./build/p1/http_static_assets.wasm ./build/http_static_assets.wasm

sign: ## Sign the build
	wash build --sign-only
