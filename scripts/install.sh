#!/bin/sh
# Copyright 2019 the Deno authors. All rights reserved. MIT license.

set -e

if ! command -v unzip >/dev/null; then
	echo "Error: unzip is required to install rgl (see: https://github.com/ink0rr/rgl#unzip-is-required)." 1>&2
	exit 1
fi

if [ "$OS" = "Windows_NT" ]; then
	target="x86_64-pc-windows-msvc"
else
	case $(uname -sm) in
	"Darwin x86_64") target="x86_64-apple-darwin" ;;
	"Darwin arm64") target="aarch64-apple-darwin" ;;
	"Linux x86_64") target="x86_64-unknown-linux-gnu" ;;
	*)
		echo "Unsupported OS + CPU combination: $(uname -sm)"
		exit 1
		;;
	esac
fi

if [ $# -eq 0 ]; then
	rgl_uri="https://github.com/ink0rr/rgl/releases/latest/download/rgl-${target}.zip"
else
	rgl_uri="https://github.com/ink0rr/rgl/releases/download/${1}/rgl-${target}.zip"
fi

bin_dir="$HOME/.rgl/bin"
exe="$bin_dir/rgl"

if [ ! -d "$bin_dir" ]; then
	mkdir -p "$bin_dir"
fi

curl --fail --location --progress-bar --output "$exe.zip" "$rgl_uri"
unzip -d "$bin_dir" -o "$exe.zip"
chmod +x "$exe"
rm "$exe.zip"

echo
echo "rgl was installed successfully to $exe"
if ! command -v rgl >/dev/null; then
	case $SHELL in
	/bin/zsh) shell_profile=".zshrc" ;;
	*) shell_profile=".bashrc" ;;
	esac
	echo export PATH=\"\$HOME/.rgl/bin:\$PATH\" >>$HOME/$shell_profile
	echo "You may need to restart the shell for the changes to take effect."
fi
echo "Run 'rgl --help' to get started"
