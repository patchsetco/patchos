SUMMARY = "Bootstrap PatchOS system image"
DESCRIPTION = "Minimal bootable image for PatchOS development"
LICENSE = "MIT"

inherit core-image

IMAGE_INSTALL:append = " \
    systemd \
    systemd-networkd \
    util-linux \
    iproute2 \
    iputils \
    procps \
"

IMAGE_FEATURES += "allow-empty-password allow-root-login empty-root-password post-install-logging tools-debug"

IMAGE_LINGUAS = "en-us"
