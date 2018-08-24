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

Hot reoading is reccommended via:

```
cargo install cargo-watch
cargo watch -x run
```

