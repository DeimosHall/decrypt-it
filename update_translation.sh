#!/usr/bin/env bash

BUILD_DIR="translation-build/"
if [ -d "$BUILD_DIR" ]; then
	rm -r translation-build
fi

meson translation-build
meson compile -C translation-build decrypt-it-pot
# meson compile -C translation-build decrypt-it-update-po

rm -r translation-build
