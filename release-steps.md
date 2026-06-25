# Updates

This document describes steps to follow to deploy a new version on Flathub.

## App version

The app version must be updated in two files:

1. [Cargo.toml](Cargo.toml)
2. [meson.build](meson.build)

## Release notes

Add release notes on the [meta info](dev.deimoshall.DecryptIt.metainfo.xml.in.in) file.

## Screenshots

Update app screenshots if required.

> New screenshots or modification of existing ones must be added/edited in the [meta info](dev.deimoshall.DecryptIt.metainfo.xml.in.in) file.

## GitHub release

When releasing a new version for flathub, I must create a source tarball that includes the vendored Rust dependencies:

2. Vendor the dependencies and setup the build directory:

```bash
meson setup build --reconfigure
```

3. Create the distribution tarball with vendored deps (takes a couple of minutes):

```bash
meson dist -C build --allow-dirty --no-tests
```

> This generates a tarball in `build/meson-dist/` named `decrypt-it-X.Y.Z.tar.xz`

6. Upload this tarball to the GitHub release

> Flatpak builds run in offline mode, so cargo must be able to find all dependencies locally in the `vendor/` directory that's bundled in the tarball. The project's `build-aux/dist-vendor.sh` script automatically handles vendoring during distribution.

## Deploy steps

After tagging the release, update the flathub manifest:

1. Go to my flathub repository:

```bash
cd ~/Projects/infra/dev.deimoshall.DecryptIt
```

1. Update the `url` in the decrypt-it module to point to my new release tarball

2. Update the `sha256` with the value from the file created by step 3

3. Commit and open a PR
