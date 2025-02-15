#!/bin/sh

PORT=${1:-4242}

# Run the server in the background
cargo run --release --bin server -- "$PORT" &
SERVER_PID=$!

# Run the client in the foreground
cargo run --release --bin client -- "127.0.0.1:$PORT"

# Kill the server after the client exits
kill $SERVER_PID
