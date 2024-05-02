ARCH := $(shell uname -m | sed 's/aarch64/arm64/')

wasi-sdk:
	docker build -t aztecprotocol/wasi-sdk:$(ARCH)-22.0 --target wasi-sdk --push .
	docker manifest create aztecprotocol/wasi-sdk:22.0 \
		--amend aztecprotocol/wasi-sdk:x86_64-22.0 \
		--amend aztecprotocol/wasi-sdk:arm64-22.0
	docker manifest push aztecprotocol/wasi-sdk:22.0

FOUNDRY_TAG := de33b6af53005037b463318d2628b5cfcaf39916
foundry:
	docker build -t aztecprotocol/foundry:$(ARCH)-$(FOUNDRY_TAG) --build-arg TAG=$(FOUNDRY_TAG) --target foundry --push .
	docker manifest create aztecprotocol/foundry:$(FOUNDRY_TAG) \
		--amend aztecprotocol/foundry:x86_64-$(FOUNDRY_TAG) \
		--amend aztecprotocol/foundry:arm64-$(FOUNDRY_TAG)
	docker manifest push aztecprotocol/foundry:$(FOUNDRY_TAG)

osxcross:
	docker build -t aztecprotocol/osxcross:$(ARCH)-14.0 --target osxcross --push .
	docker manifest create aztecprotocol/osxcross:14.0 \
		--amend aztecprotocol/osxcross:x86_64-14.0 \
		--amend aztecprotocol/osxcross:arm64-14.0
	docker manifest push aztecprotocol/osxcross:14.0

build:
	docker build -t aztecprotocol/build --target build .

devbox:
	docker build -t aztecprotocol/devbox --target devbox .

sysbox:
	docker build -t aztecprotocol/sysbox --target sysbox .

all: build devbox sysbox

.PHONY: all build devbox sysbox
