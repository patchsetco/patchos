SUMMARY = "PatchOS controller daemon"
DESCRIPTION = "A small Rust daemon that reports basic PatchOS system information"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/COPYING.MIT;md5=3da9cfbcb788c80a0384361b4de20420"

inherit cargo systemd

FILESEXTRAPATHS:prepend := "${THISDIR}/../../../services/patchd:"

SRC_URI = " \
    file://Cargo.toml \
    file://Cargo.lock \
    file://src/device.rs \
    file://src/main.rs \
    file://src/socket.rs \
    file://src/status.rs \
    file://patchd.service \
"

S = "${UNPACKDIR}"

SYSTEMD_SERVICE:${PN} = "patchd.service"

do_install:append() {
    install -D -m 0644 ${UNPACKDIR}/patchd.service \
        ${D}${systemd_system_unitdir}/patchd.service
}

FILES:${PN} += "${systemd_system_unitdir}/patchd.service"
