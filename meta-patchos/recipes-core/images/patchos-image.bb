SUMMARY = "Bootstrap PatchOS system image"
DESCRIPTION = "Minimal bootable image for PatchOS development"
LICENSE = "MIT"

inherit core-image

IMAGE_INSTALL:append = " \
    patchos-config \
    patchd \
    systemd-networkd \
    iproute2 \
    iputils \
    procps \
    util-linux \
"

IMAGE_LINGUAS = "en-us"
