.SILENT:
.PHONY: fuzz

# VS Code: Ctrl+Shift+B
all: test fmt lint build-doc

test:
	cargo test

fmt:
	cargo +nightly fmt --all -- --check || (cargo +nightly fmt --all && exit 1)

lint:
	cargo clippy --all-targets --all-features

coverage-html:
	cargo llvm-cov --html
	open target/llvm-cov/html/index.html

# VS Code:
# - Activate once: F1 -> Coverage Gutters: Watch
# - Generate coverage when needed: F1 -> Tasks: Run Task -> coverage
coverage-info:
	mkdir -p target/llvm-cov
	cargo llvm-cov --all-features --workspace --lcov --output-path target/llvm-cov/lcov.info 

bench:
	cargo bench --profile release
	xdg-open target/criterion/push\ and\ clear/report/index.html

doc:
	cargo doc --no-deps --open

build-doc:
	cargo doc --no-deps

fuzz:
	cargo +nightly fuzz run static_vector

dev:
	echo Installing pre-commit hook...
	curl -1sLf 'https://dl.cloudsmith.io/public/evilmartians/lefthook/setup.deb.sh' | sudo -E bash
	sudo apt install lefthook
	lefthook install

	echo Installing code coverage...
	host=$$(rustc -vV | grep '^host:' | cut -d' ' -f2); \
	curl --proto '=https' --tlsv1.2 -fsSL "https://github.com/taiki-e/cargo-llvm-cov/releases/latest/download/cargo-llvm-cov-$${host}.tar.gz" \
		| tar xzf - -C "$$HOME/.cargo/bin"

	echo Installing cargo-fuzz...
	cargo install cargo-fuzz
