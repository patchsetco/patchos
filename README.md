# PatchOS

**Use what you have.**

PatchOS is an operating system for turning spare computers into useful personal infrastructure.

## Status

PatchOS is in early development.

## Repository structure

```text
meta-patchos/
├── conf/
│   ├── distro/
│   │   └── patchos.conf
│   └── layer.conf
└── recipes-core/
    └── images/
        └── patchos-image.bb
```

## Build target

```bash
bitbake patchos-image
```

## Run in QEMU

```bash
runqemu patchos-image snapshot nographic
```

## License

PatchOS is open-source software licensed under the MIT License. See [LICENSE](LICENSE).

Third-party components retain their original licenses.
