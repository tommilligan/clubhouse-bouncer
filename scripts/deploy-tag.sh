#!/bin/sh
set -ev

# Login to DockerHub
docker login -u "$DOCKER_USERNAME" -p "$DOCKER_PASSWORD"

VERSION=${TRAVIS_TAG}
IMAGE_TAG="${DOCKER_REPO}:${VERSION}"

# Build image
docker tag -t "${DOCKER_REPO}" "${IMAGE_TAG}"
docker push "$IMAGE_TAG"

