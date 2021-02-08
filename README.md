This is a test case for an actix-web Json REST API server that streams
sqlx query results.

The purpose of this app is to explore issues with streaming.

To reproduce this, run the app and the test concurrently:

    cargo run --release &
    drill --stats -q --benchmark tracks.yml

