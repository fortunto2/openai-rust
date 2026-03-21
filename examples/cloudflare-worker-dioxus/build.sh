#!/bin/bash
set -e

echo "=== Building Dioxus Frontend ==="
cd frontend

# Ensure dx is installed
if ! command -v dx &> /dev/null
then
    echo "dx (dioxus-cli) could not be found. Installing..."
    cargo install dioxus-cli@0.5.0
fi

dx build --release
cd ..

echo "=== Building & Deploying Cloudflare Worker ==="
cd worker

# We can use wrangler to deploy, but for the example, we'll just run wrangler dev
# or tell the user how to run it.

echo "Build successful! To test locally:"
echo "cd worker && npx wrangler dev"
echo ""
echo "To deploy to Cloudflare:"
echo "cd worker && npx wrangler deploy"
