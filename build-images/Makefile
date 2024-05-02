ARCH := $(shell uname -m | sed 's/aarch64/arm64/')
FOUNDRY_TAG := de33b6af53005037b463318d2628b5cfcaf39916
VERSION := 1.0

wasi-sdk:
	docker build -t aztecprotocol/wasi-sdk:$(ARCH)-22.0 --target wasi-sdk --push .
	docker manifest create aztecprotocol/wasi-sdk:22.0 \
		--amend aztecprotocol/wasi-sdk:x86_64-22.0 \
		--amend aztecprotocol/wasi-sdk:arm64-22.0
	docker manifest push aztecprotocol/wasi-sdk:22.0

foundry:
	docker build -t aztecprotocol/foundry:$(ARCH)-$(FOUNDRY_TAG) --build-arg TAG=$(FOUNDRY_TAG) --target foundry --push .
	docker tag aztecprotocol/foundry:$(ARCH)-$(FOUNDRY_TAG) aztecprotocol/foundry:$(FOUNDRY_TAG)
	docker manifest rm aztecprotocol/foundry:$(FOUNDRY_TAG)
	docker manifest create aztecprotocol/foundry:$(FOUNDRY_TAG) \
		--amend aztecprotocol/foundry:x86_64-$(FOUNDRY_TAG) \
		--amend aztecprotocol/foundry:arm64-$(FOUNDRY_TAG)
	docker manifest push aztecprotocol/foundry:$(FOUNDRY_TAG)

osxcross:
	docker build -t aztecprotocol/osxcross:$(ARCH)-14.0 --target osxcross --push .
	docker manifest rm aztecprotocol/osxcross:14.0
	docker manifest create aztecprotocol/osxcross:14.0 \
		--amend aztecprotocol/osxcross:x86_64-14.0 \
		--amend aztecprotocol/osxcross:arm64-14.0
	docker manifest push aztecprotocol/osxcross:14.0

build:
	docker build -t aztecprotocol/build:$(ARCH)-$(VERSION) --target build .
	docker tag aztecprotocol/build:$(ARCH)-$(VERSION) aztecprotocol/build

build-push: build
	docker push aztecprotocol/build:$(ARCH)-$(VERSION)
	docker manifest rm aztecprotocol/build:$(VERSION)
	docker manifest create aztecprotocol/build:$(VERSION) \
		--amend aztecprotocol/build:x86_64-$(VERSION) \
		--amend aztecprotocol/build:arm64-$(VERSION)
	docker manifest push aztecprotocol/build:$(VERSION)

devbox:
	docker build -t aztecprotocol/devbox:$(ARCH)-$(VERSION) --target devbox .
	docker tag aztecprotocol/devbox:$(ARCH)-$(VERSION) aztecprotocol/devbox

devbox-push: devbox
	docker push aztecprotocol/devbox:$(ARCH)-$(VERSION)
	docker manifest rm aztecprotocol/devbox:$(VERSION)
	docker manifest create aztecprotocol/devbox:$(VERSION) \
		--amend aztecprotocol/devbox:x86_64-$(VERSION) \
		--amend aztecprotocol/devbox:arm64-$(VERSION)
	docker manifest push aztecprotocol/devbox:$(VERSION)

sysbox:
	docker build -t aztecprotocol/sysbox --target sysbox .

all: build devbox sysbox

.PHONY: all build devbox sysbox
