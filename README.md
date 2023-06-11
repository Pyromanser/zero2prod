# zero2prod

- cargo +nightly udeps
- cargo fmt
- cargo check
- cargo clippy  -- -W clippy::all -W clippy::pedantic
- cargo audit
- TEST_LOG=true cargo test health_check_works | bunyan
- cargo +nightly tarpaulin --verbose --all-features --workspace
