# zero2prod

- cargo +nightly udeps
- cargo fmt
- cargo check
- cargo clippy  -- -W clippy::all -W clippy::pedantic
- cargo audit
- TEST_LOG=true cargo test health_check_works | bunyan
- cargo +nightly tarpaulin --verbose --all-features --workspace


- cargo sqlx prepare --check -- --bin zero2prod
- cargo sqlx prepare -- --lib
- docker build --tag zero2prod --file Dockerfile .
- docker run --rm --name zero2prod -p 8000:8000 zero2prod