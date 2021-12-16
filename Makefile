#CARGO=RUSTFLAGS='-F warnings -A deprecated' cargo
CARGO=cargo


debug:
	$(CARGO) build -j 1 --all


release:
	$(CARGO) build -j 1 --all  --release

test:
	RUST_BACKTRACE=full $(CARGO) test -j 1 --all 2>&1


test-release:
	RUST_BACKTRACE=full $(CARGO) test -j 1 --release --all


bench:
	-rm target/bench.log
	cargo bench --all --no-run |tee target/bench.log
	cargo bench --all --jobs 1 |tee -a target/bench.log

fmt:
	cargo fmt --all


fmt_check:
	cargo fmt --all -- --check

cov:
	cargo cov test --all
	cargo cov report --open

clean:
	rm -rf target/debug/
	rm -rf target/release/

clippy:
	$(CARGO) clippy -j 1 --all

