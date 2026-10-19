all: build

lib:
	cd ../swift && make build

build: lib
	xcodebuild -scheme KuboXcode -destination 'platform=macOS' build

xcodebuild: build

run: build
	swift run -Xlinker -L../swift -Xlinker -lkubo_ffi \
	  -Xlinker -framework -Xlinker Security \
	  -Xlinker -framework -Xlinker CoreFoundation \
	  -Xlinker -lresolv -Xlinker -lpthread -Xlinker -ldl

clean:
	rm -rf .build .swiftpm
.PHONY: all lib build run clean