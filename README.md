This is a test case for an actix-web Json REST API server that streams
sqlx query results.

The purpose of this app is to explore issues with streaming.

To reproduce this, run the app and the test concurrently:

    cargo run --release &
    drill --stats -q --benchmark tracks.yml

and the output showing the issues is:

    [2021-02-08T05:16:07Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 2 items
    [2021-02-08T05:16:07Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 2 items
