SUMMARY = "Base configuration for PatchOS"
DESCRIPTION = "Installs PatchOS wired network configuration"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/COPYING.MIT;md5=3da9cfbcb788c80a0384361b4de20420"

SRC_URI = "file://20-wired.network"

S = "${UNPACKDIR}"

RDEPENDS:${PN} = "systemd-networkd"

do_install() {
    install -Dm0644 ${S}/20-wired.network \
        ${D}${sysconfdir}/systemd/network/20-wired.network
}

FILES:${PN} += "${sysconfdir}/systemd/network/20-wired.network"
