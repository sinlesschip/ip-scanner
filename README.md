# An IP host-up scanner

checks if host is up on all address spaces on the internet.
saves live hosts in sqlite database. 

run with cargo +nighly: 

`cargo +nighly run`

can add more threads with `--threads`:

`cargo +nightly run -- --threads 50`
