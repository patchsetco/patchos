# meta-patchos

This layer contains the OpenEmbedded metadata used to build PatchOS.

## Compatibility

Yocto Project 6.0 Wrynose

## Dependencies

* OpenEmbedded-Core

## Configuration

Add this layer to the build:

```
bitbake-layers add-layer /path/to/meta-patchos
```

Select the PatchOS distribution:

```
DISTRO = "patchos"
```

Build the PatchOS image:

```
bitbake patchos-image
```

## License

PatchOS layer metadata is licensed under the MIT License. See [COPYING.MIT](COPYING.MIT).

Upstream OpenEmbedded components retain their original licenses.

## Maintainer

Patchset Company
