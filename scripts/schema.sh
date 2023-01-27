#!/bin/bash

# When all schemas are ready, can create schemas like this:
for f in ./contracts/*
do
  cd "$f"
  CMD="cargo run --example schema"
  cargo run --example schema
  cd "$START_DIR"
done
