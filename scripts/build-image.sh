#!/bin/sh
set -ev

# build image
docker pull "${SOCKER_REPO}"
docker build --cache-from "${DOCKER_REPO}" -t "${DOCKER_REPO}" . 
