NAME := ft_lgtm
AUTHORS := mcutura

SHELL := /bin/bash
SESSION := --connect qemu:///session

VM_NAME ?= LGTM
HOSTNAME ?= lgtm-host
VM_RAM_MB ?= 8192
VM_VCPUS ?= 8
VM_DISK_SIZE_GB ?= 25

SSH_KEY ?= ${HOME}/.ssh/mc-putchar.pub
HOST_SSH_PORT ?= 2242
PORT_FORWARDING := "hostfwd=tcp::$(HOST_SSH_PORT)-:22,hostfwd=tcp::8080-:80,hostfwd=tcp::8443-:443,hostfwd=tcp::5001-:5001,hostfwd=tcp::5002-:5002"
VM_UNDEFINE_OPTS := --snapshots-metadata --remove-all-storage

MOUNT_DIR ?= ${HOME}/goinfre
ISO_DIR := $(MOUNT_DIR)/iso
VM_IMGDIR := $(MOUNT_DIR)/VMs
VM_IMG := $(VM_IMGDIR)/iot.qcow2
VM_CLOUDIMG := $(VM_IMGDIR)/iot-cloud.qcow2
USER_DATA := host/user-data

BUILD_LOC ?= host
DEPLOY_ARGS ?=
DOMAIN_URL ?= http://lgtm.localhost:8080

OS_VARIANT := ubuntu22.04
ARCH := $(shell uname -m)
ifeq ($(ARCH), x86_64)
ISO_FILE := $(ISO_DIR)/ubuntu-22.04.5-live-server-amd64.iso
ISO_URL := https://releases.ubuntu.com/22.04/ubuntu-22.04.5-live-server-amd64.iso
CLOUDIMG_FILE := $(ISO_DIR)/jammy-server-cloudimg-amd64.img
CLOUDIMG_URL := https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-amd64.img
VM_BOOT := hd,cdrom
SED_FLAG := -i
else ifeq ($(ARCH),arm64)
ISO_FILE := $(ISO_DIR)/ubuntu-22.04.5-live-server-arm64.iso
ISO_URL := https://releases.ubuntu.com/22.04/ubuntu-22.04.5-live-server-arm64.iso
CLOUDIMG_FILE := $(ISO_DIR)/jammy-server-cloudimg-arm64.img
CLOUDIMG_URL := https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-arm64.img
VM_BOOT := uefi
VM_UNDEFINE_OPTS += --nvram
SED_FLAG := -i ''
else
$(error Unsupported architecture: $(ARCH))
endif

# Colors
RED := \033[31m
GRN := \033[32m
MAG := \033[35m
MAB := \033[1;35m
CYA := \033[36m
CYB := \033[1;36m
NC  := \033[0m

.PHONY: help

help:	# Show this helpful message
	@awk 'BEGIN { FS = ":.*#"; \
	printf "$(GRN)$(NAME)$(NC)\nby: $(AUTHORS)\t@$(GRN)42 Berlin$(NC)\n\n"; \
	printf "Usage:\n\t$(CYB)make $(MAG)<target>$(NC)\n" } \
	/^[A-Za-z_0-9-]+:.*?#/ { printf "$(MAB)%-16s $(CYA)%s$(NC)\n", $$1, $$2}' \
	Makefile

.PHONY: start stop console ssh clean auto secret

auto:	# Automated install and deploy
ifeq ($(BUILD_LOC), host)
	@$(MAKE) build-imgs
	@$(MAKE) install
	@$(MAKE) deploy
	@$(MAKE) reload-imgs
else
	@$(MAKE) install
	@DEPLOY_ARGS=--vm-build $(MAKE) deploy
endif

start:	# Start Host VM
	virsh $(SESSION) start $(VM_NAME)

stop:	# Stop Host VM
	virsh $(SESSION) destroy $(VM_NAME)

console:	# Connect to Host VM console
	virsh $(SESSION) console $(VM_NAME)

ssh:	# SSH into Host VM
	ssh -p $(HOST_SSH_PORT) lgtm@localhost

clean:	# Remove Host VM and its storage
	$(info Cleaning up...)
	-rm -f $(USER_DATA)
	-virsh $(SESSION) destroy $(VM_NAME)
	-virsh $(SESSION) undefine $(VM_NAME) $(VM_UNDEFINE_OPTS)
	-ssh-keygen -f "$$HOME/.ssh/known_hosts" -R "[localhost]:$(HOST_SSH_PORT)" || true

secret:	# Print Grafana admin password
	ssh -p $(HOST_SSH_PORT) -o StrictHostKeyChecking=no lgtm@localhost \
		'kubectl -n lgtm get secret grafana -o jsonpath="{.data.admin-password}"' | \
		base64 --decode ; echo

devup:	# Start development environment
	docker compose -f app/compose.yaml up -d --build

devdown:	# Stop development environment
	docker compose -f app/compose.yaml down

.PHONY: install deploy undeploy build-imgs reload-imgs isofs

install: isofs $(VM_CLOUDIMG)	# Install VM from CloudImg
	virt-install $(SESSION) --name $(VM_NAME) \
		--memory $(VM_RAM_MB) \
		--vcpus $(VM_VCPUS) \
		--disk path=$(VM_CLOUDIMG),format=qcow2,bus=virtio \
		--disk path=host/seed.iso,device=disk,bus=virtio,readonly=on \
		--check disk_size=off \
		--filesystem $$(pwd),iot,type=mount,mode=squash \
		--boot $(VM_BOOT) \
		--os-variant $(OS_VARIANT) \
		--network none \
		--graphics none \
		--console pty,target_type=serial \
		--qemu-commandline="-netdev" \
		--qemu-commandline="user,id=net0,$(PORT_FORWARDING)" \
		--qemu-commandline="-device" \
		--qemu-commandline="virtio-net-pci,netdev=net0" \
		--import \
		--noautoconsole

deploy:	# Deploy the Kubernetes cluster
	@echo "Waiting for VM to boot and SSH to become available..."
	@while ! nc -z localhost $(HOST_SSH_PORT); do sleep 5; done
	@echo "Waiting for VM provisioning to complete..."
	@while ! ssh -p $(HOST_SSH_PORT) -o StrictHostKeyChecking=no lgtm@localhost 'bash -c k3d --version' 2>/dev/null; do sleep 5; done
	@echo "VM provisioning completed. Deploying the cluster..."
	ssh -p $(HOST_SSH_PORT) -o StrictHostKeyChecking=no lgtm@localhost 'bash -s' -- $(DEPLOY_ARGS) < tools/deploy-cluster.sh

undeploy:	# Delete the Kubernetes cluster
	ssh -p $(HOST_SSH_PORT) lgtm@localhost 'bash -c "k3d cluster delete lgtm-cluster"'

reload-imgs: | build-imgs	# Reload docker images in the cluster
ifeq ($(BUILD_LOC), host)
	docker push localhost:5001/ft-lgtm/backend:latest
	docker push localhost:5001/ft-lgtm/frontend:latest
else
	ssh -p $(HOST_SSH_PORT) -o StrictHostKeyChecking=no lgtm@localhost \
		'docker push localhost:5001/ft-lgtm/backend:latest && docker push localhost:5001/ft-lgtm/frontend:latest'
endif
	ssh -p $(HOST_SSH_PORT) -o StrictHostKeyChecking=no lgtm@localhost \
		'kubectl rollout restart deployment/backend deployment/frontend -n app'

build-imgs:	# Rebuild docker images
ifeq ($(BUILD_LOC), host)
	docker build -t localhost:5001/ft-lgtm/backend:latest ./app/backend
	docker build --build-arg PUBLIC_API_URL=http://localhost:3000/api/v1 -t localhost:5001/ft-lgtm/frontend:latest ./app/frontend
else
	ssh -p $(HOST_SSH_PORT) -o StrictHostKeyChecking=no lgtm@localhost \
		'docker build -t localhost:5001/ft-lgtm/backend:latest /mnt/app/backend && docker build --build-arg PUBLIC_API_URL=$(DOMAIN_URL)/api/v1 -t localhost:5001/ft-lgtm/frontend:latest /mnt/app/frontend'
endif

isofs:
	sed "s|<SSH_KEY>|$$(cat $(SSH_KEY))|g" host/user-data.yaml > "$(USER_DATA)"
	sed $(SED_FLAG) "s|<PASSWD_HASH>|$$(echo 'lgtm' | openssl passwd -6 -stdin)|g" "$(USER_DATA)"
	docker run --rm -v $(PWD)/host:/data alpine sh -c \
			"apk add --no-cache cdrkit && mkisofs -output /data/seed.iso -volid cidata -joliet -rock /data/user-data /data/meta-data"

$(VM_CLOUDIMG): $(CLOUDIMG_FILE) | $(VM_IMGDIR)
	qemu-img create -f qcow2 -b $(CLOUDIMG_FILE) -F qcow2 $(VM_CLOUDIMG) $(VM_DISK_SIZE_GB)G

$(CLOUDIMG_FILE): | $(ISO_DIR)
	@echo "Downloading cloud image..."
	@wget -O $@ $(CLOUDIMG_URL)

$(ISO_DIR) $(VM_IMGDIR):
	@mkdir -p $@

$(ISO_FILE): | $(ISO_DIR)
	@echo "Downloading ISO image..."
	@wget -O $@ $(ISO_URL)
