#!/bin/sh
VERSION=0.10.1
docker build . -t torhovland/rust-trunk:$VERSION
docker push torhovland/rust-trunk:$VERSION
