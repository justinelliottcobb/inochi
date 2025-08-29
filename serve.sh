#!/bin/bash

# Set default values
HOST="${HOST:-0.0.0.0}"
PORT="${PORT:-3000}"

echo "Starting Rust-based web server..."
echo "Serving WASM build from ./www directory"
echo ""
echo "Configuration:"
echo "  HOST=$HOST (set HOST env var to change)"
echo "  PORT=$PORT (set PORT env var to change)"
echo ""
echo "Examples:"
echo "  PORT=8080 ./serve.sh    # Run on port 8080"
echo "  HOST=127.0.0.1 ./serve.sh    # Bind to localhost only"
echo ""

# Export for cargo to use
export HOST
export PORT

cargo run --bin server