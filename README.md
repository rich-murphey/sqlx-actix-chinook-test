This rust app explores issues involving streaming SQL query results
via a JSON REST API.

Sqlite shows Segmentation fault (in code that advances the cursor)
under high load. To reproduce this use:

    cargo run --release &
    drill --stats -q --benchmark tracks.yml

Actix-web may drop a stream in the middle of a sequence of rows.  To
reproduce issues this, run the app and the test concurrently:

    cargo run --release --features sqlx-actix-streaming/logging &
    drill --stats -q --benchmark tracks.yml

and the output showing the issues is:

    [2021-02-08T05:16:07Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 2 items
    [2021-02-08T05:16:07Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 2 items

To double check that the full payload is being read, use wrk, as follows:

    wrk -c24 -t24 -d8s -s tracks.lua http://127.0.0.1:8080

This should report something like:

    Running 8s test @ http://127.0.0.1:8080
      24 threads and 24 connections
      Thread Stats   Avg      Stdev     Max   +/- Stdev
        Latency    59.24ms   22.83ms 138.42ms   64.87%
        Req/Sec    16.83      5.98    30.00     54.31%
      3255 requests in 8.10s, 564.37MB read

while the server reports:

    [2021-02-09T21:41:05Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 733 items
    [2021-02-09T21:41:21Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 26 items
    [2021-02-09T21:41:21Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 148 items
