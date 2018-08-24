#!/bin/sh
set -ev

# Login to DockerHub
docker login -u "$DOCKER_USERNAME" -p "$DOCKER_PASSWORD"

VERSION=${TRAVIS_TAG}
IMAGE_TAG="tommilligan/clubhouse-bouncer:${VERSION}"

# Build image
docker build -t "${IMAGE_TAG}" .
docker push $IMAGE_TAG

