#!/bin/sh
set -ev

build_refresh () {
  # update image repo/image:<tag> from Dockerfile.<tag>
  DOCKER_IMAGE="$1:$2"
  docker pull "${DOCKER_IMAGE}"
  docker build --cache-from "${DOCKER_IMAGE}" -f "Dockerfile.$2" -t "${DOCKER_IMAGE}" .
}

# pull and refresh build base/builder
# this doesn't cost anything extra as we need the images anyway
build_refresh "${DOCKER_REPO}" "base"
build_refresh "${DOCKER_REPO}" "builder"

# build actual image
docker pull "${DOCKER_REPO}"
docker build --cache-from "${DOCKER_REPO}" -t "${DOCKER_REPO}" . 

