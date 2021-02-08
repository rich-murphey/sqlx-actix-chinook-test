
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
