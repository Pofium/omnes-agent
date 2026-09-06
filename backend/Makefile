IMAGE_NAME    = omnesagent
IMAGE_TAG     = stagex
IMAGE_FAT_TAG = stagex-fat

.PHONY: build build-fat extract extract-fat shell-debug clean

build:
	podman build -t $(IMAGE_NAME):$(IMAGE_TAG) --target package -f Containerfile .

build-fat:
	podman build -t $(IMAGE_NAME):$(IMAGE_FAT_TAG) --target package-fat -f Containerfile .

extract:
	@if ! podman image exists $(IMAGE_NAME):$(IMAGE_TAG) 2>/dev/null; then \
		$(MAKE) build; \
	fi
	podman create --name omnesagent-extract $(IMAGE_NAME):$(IMAGE_TAG)
	podman cp omnesagent-extract:/usr/bin/omnesagent .
	podman cp omnesagent-extract:/usr/bin/zerocode .
	podman rm omnesagent-extract
	ls -lh omnesagent zerocode

extract-fat:
	@if ! podman image exists $(IMAGE_NAME):$(IMAGE_FAT_TAG) 2>/dev/null; then \
		$(MAKE) build-fat; \
	fi
	podman create --name omnesagent-fat-extract $(IMAGE_NAME):$(IMAGE_FAT_TAG)
	podman cp omnesagent-fat-extract:/usr/bin/omnesagent .
	podman cp omnesagent-fat-extract:/usr/bin/zerocode .
	podman rm omnesagent-fat-extract
	mv omnesagent omnesagent-fat
	ls -lh omnesagent-fat

shell-debug:
	podman run --rm -it \
		--entrypoint /bin/sh \
		docker.io/stagex/pallet-rust@sha256:abe9b95c93a5afa271f69fcd5eb18c8cd405fe5df6491a63c9418e3a170573dc

clean:
	-podman rmi $(IMAGE_NAME):$(IMAGE_TAG) $(IMAGE_NAME):$(IMAGE_FAT_TAG) 2>/dev/null
	rm -f omnesagent zerocode omnesagent-fat
