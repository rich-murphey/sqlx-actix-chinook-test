This rust app explores issues involving streaming SQL query results
via a JSON REST API.

## Dropping streams

Actix-web may intermittently drop a stream in the middle of the rows of
a query response.  To reproduce this issue, run the app and the test
concurrently:

    cargo run --release --features sqlx-actix-streaming/logging &
    drill --stats -q --benchmark tracks.yml

The corresponding server output showing the issues is:

    [2021-02-08T05:16:07Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 2 items
    [2021-02-08T05:16:07Z ERROR sqlx_actix_streaming::bytestream] dropped ByteStream in state: NonEmpty after 2 items

This prints the output of a single /tracks method call.

    ht -j POST http://localhost:8080/tracks offset:=0 limit:=100

This verifies that the complete result is being returned for every API
call.

    wrk -c24 -t24 -d4s -s tracks.lua http://localhost:8080

This prints a histogram of the API latency:

    hey -n 2048 -c 8 -H "Content-Type: application/json" -m POST -d '{"offset":0,"limit":3000}' http://localhost:8080/tracks

## Issues in SQLite

Sqlite may exhibit a Segmentation fault (in code that advances the cursor)
under high load. To reproduce this, use:

    cargo run --release &
    drill --stats -q --benchmark tracks.yml

