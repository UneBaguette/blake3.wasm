ROOT := pkg
CRATE := blake3_wasm_rs
VERSION := 0.3.0

.PHONY: all clean build bundler node web node-esm package verify

all: clean build package

clean:
	rm -rf $(ROOT)

build: bundler node web node-esm

bundler:
	@echo "Building bundler target..."
	wasm-pack build --target bundler -d $(ROOT)/bundler

node:
	@echo "Building nodejs target..."
	wasm-pack build --target nodejs -d $(ROOT)/node

web:
	@echo "Building web target..."
	wasm-pack build --target web -d $(ROOT)/web

node-esm: node
	@echo "Generating node-esm wrapper..."
	@mkdir -p $(ROOT)/node-esm
	@node -e "\
		const fs = require('fs'); \
		const dts = fs.readFileSync('$(ROOT)/node/$(CRATE).d.ts', 'utf8'); \
		const fns = [...dts.matchAll(/export function (\w+)/g)].map(m => m[1]); \
		const cls = [...dts.matchAll(/export class (\w+)/g)].map(m => m[1]); \
		const lines = [ \
			\"import { createRequire } from 'module';\", \
			\"const require = createRequire(import.meta.url);\", \
			\"const mod = require('../node/$(CRATE).js');\", \
			\"export default mod;\", \
			...fns.map(n => 'export const ' + n + ' = mod.' + n + ';'), \
			...cls.map(n => 'export const ' + n + ' = mod.' + n + ';'), \
		]; \
		fs.writeFileSync('$(ROOT)/node-esm/index.mjs', lines.join('\n') + '\n'); \
	"


package: build
	@cp README.md $(ROOT)/README.md
	@rm -f $(ROOT)/bundler/package.json $(ROOT)/node/package.json $(ROOT)/web/package.json
	@rm -f $(ROOT)/bundler/.gitignore $(ROOT)/node/.gitignore $(ROOT)/web/.gitignore
	@node -e "\
		const pkg = { \
			name: 'blake3-wasm-rs', \
			version: '$(VERSION)', \
			description: 'BLAKE3 hashing via Rust/WASM - works in Node.js (CJS + ESM), browsers, and bundlers', \
			license: 'MIT', \
			repository: { \
				type: 'git', \
				url: 'https://github.com/UneBaguette/blake3.wasm', \
			}, \
			exports: { \
				'.': { \
					node: { \
						require: './node/$(CRATE).js', \
						import: './node-esm/index.mjs', \
					}, \
					import: './bundler/$(CRATE).js', \
					default: './web/$(CRATE).js', \
				}, \
			}, \
			types: './bundler/$(CRATE).d.ts', \
			files: ['node/', 'node-esm/', 'bundler/', 'web/', 'README.md'], \
			keywords: ['blake3', 'wasm', 'hash', 'cryptography', 'wasm-bindgen'], \
		}; \
		require('fs').writeFileSync('$(ROOT)/package.json', JSON.stringify(pkg, null, 2) + '\n'); \
	"
	@echo ""
	@echo "Done! Package ready in $(ROOT)/"
	@echo "  node (CJS):   $(ROOT)/node/"
	@echo "  node (ESM):   $(ROOT)/node-esm/"
	@echo "  bundler:      $(ROOT)/bundler/"
	@echo "  web:          $(ROOT)/web/"

verify: package
	cd $(ROOT) && npm pack --dry-run

