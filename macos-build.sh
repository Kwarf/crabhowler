#!/usr/bin/env bash

cargo build --release
mkdir -p Wavy.clap/Contents/MacOS
cp target/release/libwavy.dylib Wavy.clap/Contents/MacOS/libwavy.dylib
cp Info.plist Wavy.clap/Contents/Info.plist
cp -r Wavy.clap ~/Library/Audio/Plug-Ins/CLAP/Wavy.clap
