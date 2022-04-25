.PHONY fmt:
fmt:
	cargo fmt

.PHONY test/unit:
test/unit:
	cargo test -q

.PHONY lint:
lint:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY test/e2e:
test/e2e:
	ls tests/*.csv | xargs -I @ bash -c "diff -u @_expected <(cargo run -q @)"

.PHONY test/all:
test/all:
	@${MAKE} test/unit
	@${MAKE} test/e2e
