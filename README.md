# clubhouse-bouncer

A thin logic layer over the [Clubhouse API](https://clubhouse.io/api/rest/v2/), clubhouse-bouncer acts as a proxy for complex requests.

## Environment variables

For required environment variables, see `.env.example`. The app will panic if these are not present.

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

For integration with the API, copy and use the `deployable.sh` script. You will need environment:

- `CLUBHOUSE_BOUNCER_API_KEY=<alphanumeric_secret>`
- `CLUBHOUSE_BOUNCER_URL=https://<base_hostname>`

Then run `./deployable.sh 1234` where `1234` is a valid ticket number (from a story such as `ch1234`).

### HTTP API

An example use of the app might be to test whether a series of tickets are deployable, such as:

```
curl -X GET localhost:2686/deployable -d '{"story_ids":["9812", "9813", "8810", "8812"]}' -H "Authorization: <api_key>" | jq '.deployable'
```

