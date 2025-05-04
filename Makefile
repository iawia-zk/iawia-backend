run: ./target/debug/tee-server
	./target/debug/tee-server --server-address=127.0.0.1:8888 '--database-url=postgres://postgres:mysecretpassword@localhost:5432/iawia?sslmode=disable' --circuit-folder="./circuits" --zkey-folder="./zkeys" --rapidsnark-path="./rapidsnark"

build:
	cargo build --locked --features disclose
