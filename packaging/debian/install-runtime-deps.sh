#!/usr/bin/env sh
set -eu

sudo apt-get update
sudo apt-get install -y \
  gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad \
  gstreamer1.0-libav \
  gstreamer1.0-pipewire \
  gstreamer1.0-pulseaudio \
  libwebrtc-audio-processing1 \
  xdg-desktop-portal \
  xdg-desktop-portal-hyprland
