# Surway

[![Deploy Surway](https://github.com/torhovland/surway/actions/workflows/deploy.yml/badge.svg)](https://github.com/torhovland/surway/actions/workflows/deploy.yml)

An [OpenStreetMap](https://www.openstreetmap.org/) surveyor app. Runs on https://surway.hovland.xyz.

The development progress of this app has been documented in [this blog series](https://blogg.bekk.no/building-an-openstreetmap-app-in-rust-part-i-2adf72c75229).

## Features

- Shows nearby OSM ways on a map.
- Shows OSM tags for the way nearest you.
- Editor for notes geo-located at your current position (useful for later editing).
- Uploading notes to OSM.
- Screen wake-lock; ability to keep screen on (on supported browsers/devices).

### Planned features

- OSM authentication, so notes are not posted anonymously.

- Uploading to OSM
    - GPX tracks
    - Points of interest (POI)

- Turn map position tracking on/off, including auto off when panning.

- Being able to add a geo-located note or POI anywhere on the map.

- PWA features
    - Manifest for showing the web app as a phone app.
    - Being able to reopen the app and show the previously downloaded ways while being offline.
    - Push notifications when an alert triggers (see below).

- Downloading third-party nearby notes from OSM (for surveying).

- Configurable alerts, e.g.
    - When you are no longer near a way.
    - When a major road is missing a name.

- Choosing between north up/head up.

## Building and running

Install Trunk, then run:

```
trunk serve      
```

## Contributing

Issues, discussions, and pull requests are welcome!
