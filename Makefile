
default:
	cargo run --release

drill:
	drill --stats -q --benchmark tracks.yml

run:
	cargo run

dump:
	sqlite3 data.db <<<.dump

tracks:
	ht -j GET  http://localhost:8080/tracks4/3/0
	ht -j POST http://localhost:8080/tracks offset:=0 limit:=3

track_table:
	ht -j POST http://localhost:8080/track_table offset:=0 limit:=3

hey:
	hey -n 2048 -c 8 -H "Content-Type: application/json" -m POST -d '{"offset":0,"limit":3000}' http://localhost:8080/tracks

wrk:
	wrk -c24 -t24 -d4s -s tracks.lua http://localhost:8080

db:
	sqlx db create
	curl https://raw.githubusercontent.com/lerocha/chinook-database/master/ChinookDatabase/DataSources/Chinook_MySql.sql |mysql -h hv.lan -u root -p$(shell sed -ne '/^DATABASE_URL/s/.*:\(.*\)@.*/\1/p' <.env)
	curl https://raw.githubusercontent.com/lerocha/chinook-database/master/ChinookDatabase/DataSources/Chinook_MySql_AutoIncrementPKs.sql |mysql -h hv.lan -u root -p$(shell sed -ne '/^DATABASE_URL/s/.*:\(.*\)@.*/\1/p' <.env)
