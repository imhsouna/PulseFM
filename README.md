# Pulse FM RDS Encoder

Rust port of the RDS/MPX generator with a modern `iced` UI and a CLI. It outputs an **FM multiplex (MPX)** signal to an audio device (192 kHz) or generates a **228 kHz MPX WAV** file for offline use. It **does not transmit RF**.

## What this is
- RDS group generator and 57 kHz BPSK subcarrier
- FM multiplex builder with stereo pilot and L−R subcarrier
- WAV output for offline analysis or feeding another SDR tool

## What this is not
- A transmitter. There is no GPIO/PWM/DMA RF path.
- A replacement for Raspberry Pi RF hardware transmitters.

## Build (cross-platform)

```bash
cargo build
```

## Run the UI

```bash
cargo run
```

## Run the CLI

```bash
cargo run --bin pulse-fm-rds-cli -- \
  --out mpx.wav \
  --duration 10 \
  --ps "RASP-PI" \
  --rt "Hello, world" \
  --pi 1234 \
  --audio /path/to/input.wav
```

## Releases (GitHub Actions)

Tag a release to build macOS, Windows, and Linux binaries and upload to GitHub Releases:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Artifacts are uploaded as zip files by `.github/workflows/release.yml`.

## Local release builds

```bash
./scripts/build_release.sh
```

This builds for any installed targets and places binaries under `target/<triple>/release/`.

## Notes
- Live mode requires an output device that supports **192 kHz** **float32**. The app resamples the internal 228 kHz MPX to 192 kHz.
- Input devices are optional; when selected, audio is injected into the MPX signal with RDS.
- RDS options supported: PI, PS, RT, TP, TA, PTY, MS, DI, RT A/B, CT, AF list, PS/RT scrolling.
- Output gain and limiter are available in live mode and WAV export.
- Live MPX meter includes RMS/peak, 19 kHz pilot, 57 kHz RDS, and an 8-band spectrum.
- Defaults are set for BOUZIDFM (Sidi Bouzid, 98.0 MHz). Update PI to your assigned value.
- Only WAV input is supported for file-based audio (via `hound`).
- WAV output is 32-bit float at 228 kHz, scaled to match the original `rds_wav.c` (/10).

## Legal
This encoder is experimental. Transmitting RF signals without a license is illegal in many jurisdictions. This port only generates baseband audio.
