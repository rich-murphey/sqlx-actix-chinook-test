
default:
	cargo run --release
test:
	drill --stats -q --benchmark tracks.yml

run:
	cargo run

dump:
	sqlite3 data.db <<<.dump

ht:
	ht -j POST http://localhost:8080/tracks offset:=0 limit:=100

hey:
	hey -n 2048 -c 8 -H "Content-Type: application/json" -m POST -d '{"offset":0,"limit":3000}' http://localhost:8080/tracks
