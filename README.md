# blake3 to wasm

**blake3.wasm** is a WebAssembly port of the BLAKE3 cryptographic hash function written in **Rust**. It enables fast and secure hashing right inside **browsers** and **Node.js**.

## build

```sh
# lets build it!

# For NodeJS
wasm-pack build --target nodejs --release

# For the web!
wasm-pack build --target web --release
```

SIMD is enabled via `wasm32_simd` feature in `Cargo.toml` and requires:
```toml
# .cargo/config.toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=+simd128"]
```

## Usage

```ts
import * as blake3 from 'blake3-wasm-rs';

const data = new TextEncoder().encode('hello world');
const key = new Uint8Array(32).fill(1);

// One-shot hashing
blake3.hash(data);
blake3.hashXof(data, 64); // variable output length

// MAC and key derivation
blake3.keyedHash(data, key);
blake3.deriveKey('my context', key);

// Conctruct for streaming
const hasher = new blake3.Hasher();

hasher.update(data.slice(0, 5));
hasher.update(data.slice(5));
hasher.finalize();
```

#### OR

```js
import { hash, hashXof, keyedHash, deriveKey, Hasher } from 'blake3-wasm-rs'

const data = new TextEncoder().encode('hello world')
const key = new Uint8Array(32).fill(1)

// One-shot hashing
hash(data)
hashXof(data, 64) // variable output length

// MAC and key derivation
keyedHash(data, key)
deriveKey('my context', key)

// Streaming
const h = new Hasher()
h.update(data.slice(0, 5))
h.update(data.slice(5))
h.finalize()
```

## Benchmarks

Tested on **Ryzen 7 5800X**, Node.js v24.

| Size  | @noble/hashes | awasm-noble | blake3-wasm |
|-------|---------------|-------------|-------------|
| 32 B  | 11 MB/s       | 34 MB/s     | 81 MB/s     |
| 1 KB  | 60 MB/s       | 360 MB/s    | 781 MB/s    |
| 64 KB | 49 MB/s       | 1,518 MB/s  | 2,004 MB/s  |
| 1 MB  | 51 MB/s       | 1,671 MB/s  | 1,893 MB/s  |
| 10 MB | 51 MB/s       | 1,572 MB/s  | 1,812 MB/s  |

## Security

The underlying `blake3` Rust crate targets algorithmic constant time. However, the JavaScript boundary (via napi-rs or WASM) introduces non-determinism from the V8 runtime that is outside our control. For absolute security, use the `blake3` Rust crate directly in a Rust program.

## See also

- [@noble/hashes](https://github.com/paulmillr/noble-hashes) | pure JS implementation
- [awasm-noble](https://github.com/paulmillr/awasm-noble) | auditable WASM implementation
- [blake3-napi](https://github.com/UneBaguette/blake3-napi)  | native Node.js addon, faster for large inputs
