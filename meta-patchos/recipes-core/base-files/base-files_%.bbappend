do_install:append() {
    printf "Use what you have.\n\n" >> ${D}${sysconfdir}/issue
    printf "%s %s\nUse what you have.\n" \
        "${DISTRO_NAME}" "${DISTRO_VERSION}" > ${D}${sysconfdir}/motd
}
