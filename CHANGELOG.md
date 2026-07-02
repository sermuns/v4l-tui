# Changelog

## 0.2.0

- improve preview experience
  - add status bar in bottom
  - able to toggle on/off by pressing 'p'
  - use low-latency `ffplay` flags

- improve error handling
  - switch to `anyhow`
  - try to show error in popup where possible, otherwise still panic

- add percentage input on controls by pressing numeric 0-9

## 0.1.1

- adjust release artifacts, just release binaries instead of `tar.gz`-archives.

## 0.1.0

- initial release
