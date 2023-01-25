START_DIR=$(pwd)

cd "$START_DIR/contracts/mod-balances"
cargo run --example schema > /dev/null

cd "$START_DIR/contracts/mod-dao"
cargo run --example schema > /dev/null

cd "$START_DIR/contracts/mod-generic"
cargo run --example schema > /dev/null

cd "$START_DIR/contracts/mod-nft"
cargo run --example schema > /dev/null

# When all schemas are ready, can create schemas like this:
# for f in ./contracts/*
# do
#   cd "$f"
#   CMD="cargo run --example schema"
#   cargo run --example schema
#   cd "$START_DIR"
# done
