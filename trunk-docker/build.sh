#!/bin/sh
VERSION=0.10.0
docker build . -t torhovland/rust-trunk:$VERSION
docker push torhovland/rust-trunk:$VERSION
