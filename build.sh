#!/usr/bin/env bash
set -euo pipefail

ROOT="pkg"
rm -rf "$ROOT"

echo "Building bundler target..."
wasm-pack build --target bundler -d "$ROOT/bundler"

echo "Building nodejs target..."
wasm-pack build --target nodejs -d "$ROOT/node"

echo "Building web target..."
wasm-pack build --target web -d "$ROOT/web"

# Generate node-esm wrapper from the nodejs build exports
echo "Generating node-esm wrapper..."
mkdir -p "$ROOT/node-esm"

# Extract export names from the nodejs .js file
EXPORTS=$(grep -oP '(?<=module\.exports\.)\w+' "$ROOT/node/blake3_wasm_rs.js" | sort -u || true)

# Fallback: parse from .d.ts if module.exports pattern not found
if [ -z "$EXPORTS" ]; then
  EXPORTS=$(grep -oP '(?<=export function )\w+' "$ROOT/node/blake3_wasm_rs.d.ts" | sort -u || true)
fi

# Also check for exported classes
CLASSES=$(grep -oP '(?<=export class )\w+' "$ROOT/node/blake3_wasm_rs.d.ts" | sort -u || true)

{
  echo "import { createRequire } from 'module';"
  echo "const require = createRequire(import.meta.url);"
  echo "const mod = require('../node/blake3_wasm_rs.js');"
  echo "export default mod;"

  for name in $EXPORTS; do
    echo "export const $name = mod.$name;"
  done

  for name in $CLASSES; do
    echo "export const $name = mod.$name;"
  done
} > "$ROOT/node-esm/index.mjs"

# Clean up wasm-pack generated package.json in each subfolder
rm -f "$ROOT/bundler/package.json" "$ROOT/node/package.json" "$ROOT/web/package.json"
rm -f "$ROOT/bundler/.gitignore" "$ROOT/node/.gitignore" "$ROOT/web/.gitignore"

# Generate root package.json
cat > "$ROOT/package.json" << 'EOF'
{
  "name": "blake3-wasm-rs",
  "version": "0.2.0",
  "description": "BLAKE3 hashing via Rust/WASM - works in Node.js (CJS + ESM), browsers, and bundlers",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/UneBaguette/blake3.wasm"
  },
  "exports": {
    ".": {
      "node": {
        "require": "./node/blake3_wasm_rs.js",
        "import": "./node-esm/index.mjs"
      },
      "import": "./bundler/blake3_wasm_rs.js",
      "default": "./web/blake3_wasm_rs.js"
    }
  },
  "types": "./bundler/blake3_wasm_rs.d.ts",
  "files": [
    "node/",
    "node-esm/",
    "bundler/",
    "web/"
  ],
  "keywords": [
    "blake3",
    "wasm",
    "hash",
    "cryptography",
    "wasm-bindgen"
  ]
}
EOF

echo ""
echo "Done! Package ready in $ROOT/"
echo "  node (CJS):     $ROOT/node/"
echo "  node (ESM):     $ROOT/node-esm/"
echo "  bundler:        $ROOT/bundler/"
echo "  web:            $ROOT/web/"
echo ""
echo "Verify with: cd $ROOT && npm pack --dry-run"