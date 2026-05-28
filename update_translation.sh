#!/usr/bin/env bash

BUILD_DIR="translation-build/"
if [ -d "$BUILD_DIR" ]; then
	rm -r translation-build
fi

meson translation-build
meson compile -C translation-build decrypteet-pot
# meson compile -C translation-build decrypteet-update-po

rm -r translation-build
