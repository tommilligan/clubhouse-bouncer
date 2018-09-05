# clubhouse-bouncer

A thin logic layer over the [Clubhouse API](https://clubhouse.io/api/rest/v2/), clubhouse-bouncer acts as a proxy for complex requests.

## Deployment

Download and run the docker image from [Docker Hub](https://hub.docker.com/r/tommilligan/clubhouse-bouncer/tags/)

### Environment variables

For required environment variables, see `.env.example`. The app will panic if these are not present.

| Variable            | Default | Description                                                  |
| ------------------- | ------- | ------------------------------------------------------------ |
| *Required*          |         |                                                              |
| BOUNCER_CREDENTIALS |         | Comma delimited alphanumeric strings, each of which is a valid `Authorization` header for API requests. Example: ```foo,bar,qux``` allows `Authorization: foo` requests. |
| CLUBHOUSE_API_TOKEN |         | Required to make authorised calls to your Clubhouse account. See `https://app.clubhouse.io/<your_organisataion_name>/settings/account/api-tokens` |
|                     |         |                                                              |
| *Respected*         |         |                                                              |
| ADDRESS             | 0.0.0.0 | Server binding address                                       |
| PORT                | 2686    | Server binding port                                          |
| RUST_LOG            |         | Logging output control. A sane choice is `bouncer=info`      |


## Development

During development, you may wish to set

```
export RUST_LOG=bouncer=trace
```

to see logs.

Hot reloading is reccommended via:

```
cargo install cargo-watch
cargo watch -x run
```

For tested rust versions, see `.travis.yml`. Docker image rust version is currently `1.26` (alpine `3.8`).

## Use

### Shell scripts

For integration with the API, copy and use the `deployable.sh` script. You will need the environment:

- `CLUBHOUSE_BOUNCER_API_KEY=<alphanumeric_secret>`
  - set with `BOUNCER_CREDENTIALS`, see above
- `CLUBHOUSE_BOUNCER_URL=https://<base_hostname>`

Then run `./deployable.sh 1234` where `1234` is a valid ticket number (from a story such as `ch1234`).

### HTTP API

An example use of the app might be to test whether a series of tickets are deployable, such as:

```
curl -X GET localhost:2686/deployable -d '{"story_ids":["9812", "9813", "8810", "8812"]}' -H "Authorization: <api_key>" | jq '.deployable'
```

## Docs

Generate openapi documentation as follows:

```
docker run --rm -v ${PWD}:/local openapitools/openapi-generator-cli generate \                                                                                                                                               ✔ 
    -i /local/openapi.yml \
    -l html2 \
    -o /local/out
```

