<div align="center">
<h1>Decrypteet</h1>

Edit file's metadata.

<img src="data/resources/icons/hicolor/512x512/apps/dev.deimoshall.Decrypteet.png" width="128" height="128" alt="Decrypteet icon">

[![Installs](https://img.shields.io/flathub/downloads/dev.deimoshall.Decrypteet?style=for-the-badge)](https://flathub.org/apps/details/dev.deimoshall.Decrypteet)
[![Latest Tag](https://img.shields.io/github/v/tag/deimoshall/Decrypteet?sort=date&style=for-the-badge)](https://github.com/deimoshall/Decrypteet/-/tags)
[![License](https://img.shields.io/github/license/deimoshall/Decrypteet?style=for-the-badge)](https://github.com/deimoshall/Decrypteet/-/raw/main/LICENSE)

</div>

## Installation
<a href='https://flathub.org/apps/details/dev.deimoshall.Decrypteet'><img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?svg&locale=en'/></a>

## About

Decrypteet is designed to help you edit exif metadata in a simple and reliable way.

<img src="data/resources/screenshots/0.png" alt="View of Decrypteet">

Decrypteet supports editing the following datatypes:

- [x] **Images:**
  - [x] JPEG, JPG, JPE
  - [x] PNG
  - [x] TIFF, TIF
  - [ ] GIF
  - [ ] BMP, DIB
  - [x] HEIC, HEIF, HIF
  - [ ] AVIF
  - [x] WebP
- [ ] **RAW formats:** CR2, CR3, NEF, ARW, RAF, ORF, RW2, DNG, and more
- [ ] **Videos:** MOV, MP4, M4V, AVI, WebM
- [ ] **Audio:** MP3, M4A, FLAC, OGG
- [ ] **Documents:** PDF
- [ ] **XMP sidecar** files

You can also drag and drop as well as paste the copied image into the app!

## Contributing

Issues and merge requests are more than welcome. However, please take the following into consideration:

- This project follows the [GNOME Code of Conduct](https://conduct.gnome.org/)
- Only Flatpak is supported

## Development

### GNOME Builder

The recommended method is to use GNOME Builder:

1. Install [GNOME Builder](https://apps.gnome.org/app/org.gnome.Builder/) from Flathub
2. Open Builder and select "Clone Repository..."
3. Clone `https://github.com/DeimosHall/Decrypteet.git` (or your fork)
4. Press "Run Project" (▶) at the top, or `Ctrl`+`Shift`+`[Spacebar]`.

### Flatpak

You can install Decrypteet from the latest commit:

1. Install [`org.flatpak.Builder`](https://github.com/flathub/org.flatpak.Builder) from Flathub
2. Clone `https://github.com/DeimosHall/Decrypteet.git` (or your fork)
3. Install the app using `flatpak-builder`:

```bash
flatpak-builder --force-clean --user --install builddir dev.deimoshall.Decrypteet.json
```

4. Run the app

```bash
flatpak run dev.deimoshall.Decrypteet
```

### Meson

You can build and install on your host system by directly using the Meson buildsystem:

1. Install `blueprint-compiler`
2. Run the following commands (with `/usr` prefix):

```
meson --prefix=/usr build
ninja -C build
sudo ninja -C build install
```

## License

This project is licensed under the GPLv3 license. See the [License](LICENSE) file for more information.

## Credits

Made with ♥️ by Deimos Hall.

Based on [`Switcheroo`](https://gitlab.com/adhami3310/Switcheroo.git) by Khaleel Al-Adhami, an app to convert and manipulate images.

This app uses [ExifTool](https://exiftool.org/) under the hood to perform metadata edits.
