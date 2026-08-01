# PatchOS

**Use what you have.**

PatchOS is an operating system for turning spare computers into useful personal infrastructure.

## Status

PatchOS is in early development.

## Build from a fresh clone

PatchOS targets Yocto Project 6.0 Wrynose and builds for `qemux86-64`.
You need a supported Linux host with Git, Python 3, and QEMU installed.

```bash
git clone https://github.com/patchsetco/patchos.git
cd patchos

./scripts/setup
./scripts/build
./scripts/run
```

The setup script downloads the pinned BitBake version, creates the Wrynose
build environment, and fetches OpenEmbedded-Core. It stores generated
workspaces and build output under `bitbake-builds/`.

## Repository structure

```text
meta-patchos/
├── conf/
│   ├── distro/
│   │   └── patchos.conf
│   ├── fragments/
│   │   └── distro/
│   │       └── patchos.conf
│   └── layer.conf
└── recipes-core/
    ├── base-files/
    │   └── base-files_%.bbappend
    ├── images/
    │   └── patchos-image.bb
    └── patchos-config/
        ├── files/
        │   └── 20-wired.network
        └── patchos-config_0.1.bb
setup/
├── patchos-build.conf
└── patchos-wrynose.conf.json
scripts/
├── build
├── run
└── setup
```

## License

PatchOS is licensed under the MIT License. See [LICENSE](LICENSE).

Third-party components retain their original licenses.
