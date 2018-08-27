#!/bin/sh
set -ev

# Push to DockerHub
docker login -u "$DOCKER_USERNAME" -p "$DOCKER_PASSWORD"
docker push "${DOCKER_REPO}"

# Deploy to Heroku
curl https://cli-assets.heroku.com/install.sh | sh
heroku --version
heroku container:login
docker tag app registry.heroku.com/clubhouse-bouncer/web
docker push registry.heroku.com/clubhouse-bouncer/web
heroku container:release -a clubhouse-bouncer web

