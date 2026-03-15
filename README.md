# boop - an online grocery store

Project for the course D0018E at Luleå University of Technology.

## Disclaimer

This is a toy project. Our handling of passwords and other sensitive data is **not secure**. Do not enter any real personal information. We do not manage real money or even simulate actual payments.

## Key features

- User-created products: just register a vendor account and create listings.
- Special offers: fine-grained control over types of discounts, scheduling, per-customer limits.
- Ratings, reviews & comments: find the best products.
- Search bar: find products by name, category or description.
- SSR: fast load times.

## Running it yourself

### Create a database

We use a PostgreSQL database. Set up a database following the schema provided in `schema.sql`. Set the `DATABASE_URL` environment variable or add it to a `.env` file in the project root or any parent directory.

### Install Rust

boop is written entirely in the Rust programming language. Install Rust along with the Cargo package manager from [the official website](https://rust-lang.org/tools/install/).

### Install the Dioxus CLI

The Dioxus fullstack framework creates endpoints, handles SSR and creates WASM bundles. Download a [pre-compiled binary](https://dioxuslabs.com/learn/0.7/getting_started/) or build from source using `cargo install dioxus-cli`.

### Serve

Start the server using `dx run`. Potentially useful flags:
- `--addr`, `--port`: specify where the server runs.
- `--open`: open the home page in the default browser.
- `--release`: build in release (optimized) mode.

Alternatively, use `dx serve` to use hot-reloading.
