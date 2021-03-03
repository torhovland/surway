#!/bin/sh
VERSION=0.8.3
docker build . -t torhovland/rust-trunk:$VERSION
docker push torhovland/rust-trunk:$VERSION
