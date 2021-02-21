#!/bin/sh
trunk build
ls -lh dist
trunk --config Trunk-release.toml build
ls -lh dist
