.PHONY: debug, dev-doc
dev-doc:
	cargo rustdoc -- --no-defaults --passes "collapse-docs" --passes "unindent-comments"
debug: generate
	DEBUG=t cargo run
generate:
	protoc --rust_out=src/grpc/ --grpc_out=src/grpc/ --plugin=protoc-gen-grpc=/data/bin/grpc_rust_plugin src/grpc/example.proto 
