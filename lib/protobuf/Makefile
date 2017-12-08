.PHONY: generate

PROTO_FILES := backend.proto peer.proto
PREFIX_PROTO_FILES := $(addprefix "src/", $(PROTO_FILES))

generate:
	protoc --rust_out=src/ --grpc_out=src/ --plugin=protoc-gen-grpc=/data/bin/grpc_rust_plugin $(PREFIX_PROTO_FILES)
