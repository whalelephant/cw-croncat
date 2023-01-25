START_DIR=$(pwd)

echo "generating schema for CronCat"
cd packages/croncat-sdk-agents
cargo run --example schema > /dev/null

cd "$START_DIR"

for f in contracts/*
do
  echo "generating schema for ${f##*/}"
  cd "$f"
  CMD="cargo run --example schema"
  eval $CMD > /dev/null
  cd "$START_DIR"
done
