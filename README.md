# Surway

[![Deploy Surway](https://github.com/torhovland/surway/actions/workflows/deploy.yml/badge.svg)](https://github.com/torhovland/surway/actions/workflows/deploy.yml)

An [OpenStreetMap](https://www.openstreetmap.org/) surveyor app. Runs on https://surway.hovland.xyz.

The development progress of this app has been documented in [this blog series](https://blogg.bekk.no/building-an-openstreetmap-app-in-rust-part-i-2adf72c75229).

## Features

- Shows nearby OSM ways on a map.
- Shows OSM tags for the way nearest you.
- Being able to write a note geo-located at your current position (useful for later editing).

### Planned features

- Managing notes.

- Uploading to OSM
    - Geo-located notes
    - GPX tracks
    - Points of interest (POI)

- Being able to add a geo-located note or POI anywhere on the map.

- PWA features
    - Manifest for showing the web app as a phone app.
    - Being able to reopen the app and show the previously downloaded ways while being offline.
    - Screen wake-lock; ability to keep screen on.
    - Push notifications when an alert triggers (see below).

- Configurable alerts, e.g.
    - When you are no longer near a way.
    - When a major road is missing a name.

## Contributing

Issues and pull requests are welcome!
