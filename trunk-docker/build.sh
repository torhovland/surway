#!/bin/sh
VERSION=0.9.2
docker build . -t torhovland/rust-trunk:$VERSION
docker push torhovland/rust-trunk:$VERSION
