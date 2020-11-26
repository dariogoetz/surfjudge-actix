# Installation of dev system

1. Install docker
2. Install docker-compose
3. Run `sudo docker-compose up -d` in top directory
4. Fill database with test data `cat pgdumpfile | sudo docker exec -i surfjudge-actix_postgres_1 psql -U postgres`
5. Install rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
6. Install version 1.45.2 of the rust ecosystem (newer versions run in an infinite loop during compilation) `rustup toolchain install 1.45.2`
7. Build the project and run `cargo +1.45.2 run`

# Installation of dev system for generating statically linked executable
1. Perform installation of dev system
2. Install musl, e.g. `pacman -S musl`
3. Install musl target for rust `rustup target add x86_64-unknown-linux-musl --toolchain=1.45.2`
4. Build the executable `cargo +1.45.2 build --release --target=x86_64-unknown-linux-musl`
