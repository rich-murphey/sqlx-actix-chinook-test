This rust app explores issues involving streaming SQL query results
via a JSON REST API.

# Dropping streams

Actix-web may intermittanly drop a stream in the middle of the rows of
a query response.  To reproduce issues this, run the app and the test
concurrently:

    cargo run --release --features sqlx-actix-streaming/logging &
    drill --stats -q --benchmark tracks.yml

and the server output showing the issues is:

    [2021-02-08T05:16:07Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 2 items
    [2021-02-08T05:16:07Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 2 items

To double check that the full payload is being read, use wrk, as follows:

    wrk -c24 -t24 -d8s -s tracks.lua http://127.0.0.1:8080

To view the output of the test query:

    ht -j POST http://localhost:8080/tracks offset:=0 limit:=100

To look at a histogram of latency:

    hey -n 2048 -c 8 -H "Content-Type: application/json" -m POST -d '{"offset":0,"limit":3000}' http://localhost:8080/tracks

# Issues in sqlite

Sqlite may exibit a Segmentation fault (in code that advances the cursor)
under high load. To reproduce this use:

    cargo run --release &
    drill --stats -q --benchmark tracks.yml

