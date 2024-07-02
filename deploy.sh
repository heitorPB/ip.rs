#/usr/bin/env bash

set -ex

# flyctl auth login
# flyctl auth docker

nix build .#ociImage
podman image load -i ./result
podman push registry.fly.io/iprs:latest
flyctl deploy
