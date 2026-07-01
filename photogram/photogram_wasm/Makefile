WASM_PACK = wasm-pack build --dev --target web
WASM_PACK = WASM_PACK_WASM_OPT=false wasm-pack build --release --target web
WASM_PACK = WASM_PACK_WASM_OPT=true wasm-pack build --dev --target web
WASM_PACK = wasm-pack build --dev --target web
WASM_PACK = WASM_PACK_WASM_OPT=true wasm-pack build --release --target web
JS_LIB = ../../../js_lib
TS = ./typescript/

.PHONY: all
all:
	${WASM_PACK}
	$(MAKE) js

start_server:
	(cd ..; cargo run -p image_server --batch scripts/image_server.bat)

help:
	@echo "To compile the typescript to the 'js' directory (where it is checked into git):"
	@echo "    make js"
	@echo
	@echo "To install 'tsc' first install node (sad face): node-v24.14.1.pkg"
	@echo "Then"
	@echo "    make install_tsc"
	@echo ""
	@echo
	@echo "To run the image-handling server use"
	@echo "    make start_server"
	@echo
	@echo "    make upgrade_from_js_lib"


.PHONY: install_tsc
install_tsc:
	npm install typescript

.PHONY: upgrade_from_js_lib
upgrade_from_js_lib:
	cp ${JS_LIB}/typescript/html.ts ${TS}
	cp ${JS_LIB}/typescript/log.ts ${TS}
	cp ${JS_LIB}/typescript/mouse.ts ${TS}
	cp ${JS_LIB}/typescript/storage.ts ${TS}
	cp ${JS_LIB}/typescript/tabs.ts ${TS}
	cp ${JS_LIB}/typescript/zoomed_window.ts ${TS}
	cp ${JS_LIB}/typescript/wasm_memory.ts ${TS}

js:
	npx tsc -b

.PHONY: zip
zip: photogram_http.zip

photogram_http.zip: http pkg javascript
	rm star_catalog.zip

	zip star_catalog.zip favicon.ico
	zip star_catalog.zip http/Blue_Marble_2002_x10.jpg
	zip star_catalog.zip http/main.css
	zip star_catalog.zip http/favicon.ico
	zip star_catalog.zip http/index.html

	zip star_catalog.zip javascript/*.js

	zip star_catalog.zip pkg/package.json
	zip star_catalog.zip pkg/star_catalog_wasm.d.ts
	zip star_catalog.zip pkg/star_catalog_wasm.js
	zip star_catalog.zip pkg/star_catalog_wasm_bg.wasm
	zip star_catalog.zip pkg/star_catalog_wasm_bg.wasm.d.ts
