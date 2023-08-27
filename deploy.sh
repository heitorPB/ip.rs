#/usr/bin/env bash

set -ex

nix build .#ociImage
podman image load -i ./result
podman push registry.fly.io/iprs:latest
flyctl deploy
