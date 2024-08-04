# An IP scanner in Rust 

checks if host is up on all address spaces on the internet.
saves live hosts in sqlite database. 

can also run vulinerability scans on each address but this feature isnt fully complete. 

run with cargo +nighly: 

`cargo +nighly run`

can add more threads with `--threads`:

`cargo +nightly run -- --threads 50`
