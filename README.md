# Yellow Pitaya [![.github/workflows/ci.yml](https://github.com/yellow-pitaya/frontend/actions/workflows/ci.yml/badge.svg)](https://github.com/yellow-pitaya/frontend/actions/workflows/ci.yml) [![pipeline status](https://gitlab.com/yellow-pitaya/frontend/badges/main/pipeline.svg)](https://gitlab.com/yellow-pitaya/frontend/-/commits/main)

This is a desktop interface for [redpitaya](https://redpitaya.com/).

![](screenshot.png)

## Install

```
cargo build
```

## Configuration

Enable SCPI server on redpitaya : http://rp-xxxxxx.local/scpi_manager/

Or via ssh:

```
systemctl start redpitaya_scpi.service
```

You can permantly enable it on startup:

```
systemctl enable redpitaya_scpi.service
```

This will disable web applications.

## Usage

```
cargo run rp-xxxxxx.local:5000
```
