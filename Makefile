ROOT := pkg
CRATE := blake3_wasm_rs
VERSION := 0.4.0

CARGO := cargo
WASM_BINDGEN := wasm-bindgen
WASM_OPT := wasm-opt
WASM_OPT_FLAGS := --enable-bulk-memory --enable-nontrapping-float-to-int -O
WASM_TARGET := wasm32-unknown-unknown
TARGET_DIR := target/$(WASM_TARGET)/release

.PHONY: all clean build build-all node-esm package verify

all: clean build package

clean:
	rm -rf $(ROOT)

build: build-all node-esm

build-all:
	@echo "Building all target..."
	RUSTFLAGS='-C opt-level=s' $(CARGO) build --target $(WASM_TARGET) --release --features talc
	@mkdir -p $(ROOT)/bundler $(ROOT)/web $(ROOT)/node
	$(WASM_BINDGEN) --target bundler --out-dir $(ROOT)/bundler $(TARGET_DIR)/$(CRATE).wasm
	$(WASM_BINDGEN) --target web --out-dir $(ROOT)/web $(TARGET_DIR)/$(CRATE).wasm
	$(WASM_BINDGEN) --target nodejs --out-dir $(ROOT)/node $(TARGET_DIR)/$(CRATE).wasm
	@mv $(ROOT)/bundler/$(CRATE)_bg.wasm $(ROOT)/$(CRATE)_bg.wasm
	@rm -f $(ROOT)/web/$(CRATE)_bg.wasm $(ROOT)/node/$(CRATE)_bg.wasm
	$(WASM_OPT) $(WASM_OPT_FLAGS) $(ROOT)/$(CRATE)_bg.wasm -o $(ROOT)/$(CRATE)_bg.wasm
	@node -e "['bundler','web','node'].forEach(d=>{const f='$(ROOT)/'+d+'/$(CRATE).js';require('fs').writeFileSync(f,require('fs').readFileSync(f,'utf8').replace(/$(CRATE)_bg\.wasm/g,'../$(CRATE)_bg.wasm'))})"
	@rm -f $(ROOT)/bundler/package.json $(ROOT)/web/package.json
	@rm -f $(ROOT)/bundler/.gitignore $(ROOT)/web/.gitignore $(ROOT)/node/.gitignore
	@echo '{"type":"commonjs"}' > $(ROOT)/node/package.json
	@cp scripts/tpl/index.js.template $(ROOT)/index.js
	@cp scripts/tpl/index.d.ts $(ROOT)/index.d.ts
	@cp scripts/tpl/README.md $(ROOT)/README.md
	@cp LICENSE $(ROOT)/LICENSE
	@sed -i 's|// @ts-nocheck|// Types|' $(ROOT)/index.d.ts
	@node -e "\
	   const pkg = {\
	      name: 'blake3-wasm-rs',\
	      type: 'module',\
	      version: '$(VERSION)',\
	      description: 'BLAKE3 hashing via Rust/WASM - works in Node.js (CJS + ESM), browsers, and bundlers',\
	      license: 'MIT',\
	      repository: { type: 'git', url: 'https://github.com/UneBaguette/blake3.wasm' },\
	      main: 'index.js',\
	      types: 'index.d.ts',\
	      exports: { '.': {\
	         node: { require: './node/$(CRATE).js', import: './node-esm/index.mjs' },\
	         import: './bundler/$(CRATE).js',\
	         default: './web/$(CRATE).js'\
	      }},\
	      files: ['bundler/', 'web/', 'node/', 'node-esm/', '$(CRATE)_bg.wasm', 'README.md', 'LICENSE'],\
	      keywords: ['blake3', 'wasm', 'hash', 'cryptography', 'wasm-bindgen']\
	   };\
	   require('fs').writeFileSync('./$(ROOT)/package.json', JSON.stringify(pkg, null, 2) + '\n');"

node-esm:
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


package:
	@echo ""
	@echo "Done! Package ready in $(ROOT)/"
	@echo "  node (CJS):   $(ROOT)/node/"
	@echo "  node (ESM):   $(ROOT)/node-esm/"
	@echo "  bundler:      $(ROOT)/bundler/"
	@echo "  web:          $(ROOT)/web/"

verify: package
	cd $(ROOT) && npm pack --dry-run

