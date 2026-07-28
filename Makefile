ifeq ($(shell uname -m), arm64)
	ARCH := _arm64
else
	ARCH :=
endif
OS := $(shell uname -s)

# When src-tauri/modules/ contains staged binaries (e.g. aw-awatcher, aw-sync
# prepared by the activitywatch bundle build), inject them via bundle.resources
# so deb/rpm/AppImage are self-contained. Standalone builds without that dir are
# unchanged. See ActivityWatch/aw-tauri#232.
build: prebuild
ifeq ($(OS),Linux)
	@if [ -d src-tauri/modules ] && ls src-tauri/modules/aw-* >/dev/null 2>&1; then \
		echo "Bundling modules from src-tauri/modules/ into Linux packages"; \
		npm run tauri build -- --config '{"bundle":{"resources":{"modules/":"modules/"}}}'; \
	else \
		npm run tauri build; \
	fi
else
	npm run tauri build
endif

dev: prebuild
	npm run tauri dev

%/.git:
	git submodule update --init --recursive

src-tauri/icons/icon.png: aw-webui/.git
	mkdir -p src-tauri/icons
	npm run tauri icon "./aw-webui/media/logo/logo.png"

aw-webui/dist: aw-webui/.git
	cd aw-webui && make build

prebuild: aw-webui/dist node_modules src-tauri/icons/icon.png

precommit: format check

format:
	cd src-tauri && cargo fmt

check:
	cd src-tauri && cargo check && cargo clippy

package:
ifeq ($(OS),Linux)
	rm -rf target/package/aw-tauri
	mkdir -p target/package/aw-tauri
	cp src-tauri/target/release/bundle/deb/*.deb target/package/aw-tauri/aw-tauri$(ARCH).deb
	cp src-tauri/target/release/bundle/rpm/*.rpm target/package/aw-tauri/aw-tauri$(ARCH).rpm
	cp src-tauri/target/release/bundle/appimage/*.AppImage target/package/aw-tauri/aw-tauri$(ARCH).AppImage

	mkdir -p dist/aw-tauri
	rm -rf dist/aw-tauri/*
	cp target/package/aw-tauri/* dist/aw-tauri/
else
	rm -rf target/package
	mkdir -p target/package
	cp src-tauri/target/release/aw-tauri target/package/aw-tauri

	mkdir -p dist
	find dist/ -maxdepth 1 -type f -delete 2>/dev/null || true
	cp target/package/* dist/
endif

node_modules: package-lock.json
	npm ci
