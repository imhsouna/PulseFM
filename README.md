# Pulse FM RDS Encoder

A modern, cross‑platform Rust UI + CLI for generating **FM multiplex (MPX)** audio with **RDS**. It can stream live MPX to an audio device (192 kHz) or export a **228 kHz MPX WAV**. It **does not transmit RF**.

## Highlights
- Live MPX output (192 kHz float32) with meters and spectrum.
- Full RDS: PI, PS, RT, TP, TA, PTY, MS, DI, RT A/B, CT, AF list, PS/RT scrolling.
- Processing: gain, limiter, stereo separation, pre‑emphasis, compressor.
- WAV export: 228 kHz float MPX for analysis or further processing.
- RadioDNS helper & automation (SI.xml + logos + DNS records).

## Quick Start

```bash
cargo build
cargo run
```

In the UI:
1. Select an output device that supports **192 kHz float32**.
2. Configure PS/RT/PI and other RDS settings.
3. Start streaming.

## CLI

```bash
cargo run --bin pulse-fm-rds-cli -- \
  --out mpx.wav \
  --duration 10 \
  --ps "RASP-PI" \
  --rt "Hello, world" \
  --pi 1234 \
  --audio /path/to/input.wav
```

## RadioDNS (Station Logo Automation)
The app can generate a **RadioDNS pack** in `./radiodns/`:
- `SI.xml` prefilled with PS/RT, bearer, and logo URLs
- required PNG logos (or resized from your source image)
- DNS helper strings (CNAME + SRV)

In the **RadioDNS** tab:
1. Set **Base URL** (your web domain)
2. Optional: choose **Logo source**
3. Set **Broadcaster FQDN**, **SRV host**, **Port**
4. Click **Generate RadioDNS Pack**
5. Use **Validate Pack** to verify sizes

Expected hosting:
- `SI.xml` at `/radiodns/spi/3.1/SI.xml`
- logos in `/radiodns/logos/`

## macOS App Bundle
Releases include a `PulseFM.app` bundle so you get a clean launch without a terminal popup.

## Releases (GitHub Actions)
Tagging creates macOS (arm64), Windows, and Linux artifacts:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Artifacts are uploaded by `.github/workflows/release.yml`.

## Local Release Builds

```bash
./scripts/build_release.sh
```

## Troubleshooting
- **No audio devices**: click **Refresh** in the Audio tab.
- **No output**: your device must support **192 kHz float32**.
- **RadioDNS validation fails**: verify logo sizes and `SI.xml` path.

## Credits
**Developer**: Hsouna Zinoubi

**Station**: BOUZIDFM

**Contact**: imhsouna@gmail.com

## Legal
This encoder is experimental. Transmitting RF signals without a license is illegal in many jurisdictions. This project only generates baseband audio.
