NAME := ft_lgtm
AUTHORS := mcutura

SHELL := /bin/bash
SESSION := --connect qemu:///session

VM_NAME := LGTM
HOSTNAME := lgtm-host
VM_RAM_MB := 8192
VM_VCPUS := 8
VM_DISK_SIZE_GB := 25

SSH_KEY := ${HOME}/.ssh/mc-putchar.pub
HOST_SSH_PORT := 2242
PORT_FORWARDING := "hostfwd=tcp::$(HOST_SSH_PORT)-:22,hostfwd=tcp::8080-:8080"

MOUNT_DIR := ${HOME}/goinfre
ISO_DIR := $(MOUNT_DIR)/iso
VM_IMGDIR := $(MOUNT_DIR)/VMs
VM_IMG := $(VM_IMGDIR)/iot.qcow2
VM_CLOUDIMG := $(VM_IMGDIR)/iot-cloud.qcow2
USER_DATA := host/user-data

OS_VARIANT := ubuntu22.04
ARCH := $(shell uname -m)
ifeq ($(ARCH), x86_64)
ISO_FILE := $(ISO_DIR)/ubuntu-22.04.5-live-server-amd64.iso
ISO_URL := https://releases.ubuntu.com/22.04/ubuntu-22.04.5-live-server-amd64.iso
CLOUDIMG_FILE := $(ISO_DIR)/jammy-server-cloudimg-amd64.img
CLOUDIMG_URL := https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-amd64.img
else ifeq ($(ARCH),arm64)
ISO_FILE := $(ISO_DIR)/ubuntu-22.04.5-live-server-arm64.iso
ISO_URL := https://releases.ubuntu.com/22.04/ubuntu-22.04.5-live-server-arm64.iso
CLOUDIMG_FILE := $(ISO_DIR)/jammy-server-cloudimg-arm64.img
CLOUDIMG_URL := https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-arm64.img
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

.PHONY: start stop console ssh clean install

start:	# Start Host VM
	virsh $(SESSION) start $(VM_NAME)

stop:	# Stop Host VM
	virsh $(SESSION) destroy $(VM_NAME)

console:	# Connect to Host VM console
	virsh $(SESSION) console $(VM_NAME)

ssh:	# SSH into Host VM
	ssh -p $(HOST_SSH_PORT) -i $(SSH_KEY) ubuntu@localhost

clean:	# Remove Host VM and its storage
	$(info Cleaning up...)
	-rm -f $(USER_DATA)
	-virsh $(SESSION) destroy $(VM_NAME)
	-virsh $(SESSION) undefine $(VM_NAME) --snapshots-metadata --remove-all-storage
	-ssh-keygen -f "/home/${USER}/.ssh/known_hosts" -R "[localhost]:$(HOST_SSH_PORT)"

install: isofs $(VM_CLOUDIMG)	# Install VM from CloudImg
	virt-install $(SESSION) --name $(VM_NAME) \
		--memory $(VM_RAM_MB) \
		--vcpus $(VM_VCPUS) \
		--disk path=$(VM_CLOUDIMG),format=qcow2,bus=virtio \
		--disk path=host/seed.iso,device=cdrom,bus=sata \
		--filesystem $$(pwd),iot,type=mount,mode=squash \
		--boot hd,cdrom \
		--os-variant $(OS_VARIANT) \
		--network none \
		--graphics none \
		--qemu-commandline="-netdev" \
		--qemu-commandline="user,id=net0,$(PORT_FORWARDING)" \
		--qemu-commandline="-device" \
		--qemu-commandline="virtio-net-device,netdev=net0" \
		--console pty,target_type=serial \
		--import \
		--noautoconsole

isofs:
	sed "s|<SSH_KEY>|$$(cat $(SSH_KEY))|g" host/user-data.yaml > "$(USER_DATA)"
	sed -i '' "s|<PASSWD_HASH>|$$(openssl passwd -6)|g" "$(USER_DATA)"
	docker run --rm -v $(PWD)/host:/data alpine sh -c \
			"apk add --no-cache cdrkit && mkisofs -output /data/seed.iso -volid cidata -joliet -rock /data/user-data /data/meta-data"

$(VM_CLOUDIMG): $(CLOUDIMG_FILE) | $(VM_IMGDIR)
	qemu-img create -f qcow2 -b $(CLOUDIMG_FILE) -F qcow2 $(VM_CLOUDIMG) $(VM_DISK_SIZE_GB)

$(CLOUDIMG_FILE): | $(ISO_DIR)
	@echo "Downloading cloud image..."
	@wget -O $@ $(CLOUDIMG_URL)

$(ISO_DIR) $(VM_IMGDIR):
	@mkdir -p $@

$(ISO_FILE): | $(ISO_DIR)
	@echo "Downloading ISO image..."
	@wget -O $@ $(ISO_URL)
