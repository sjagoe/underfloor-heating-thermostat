test:
	cargo test --exclude underfloor-heating --workspace --target x86_64-unknown-linux-gnu

run:
	cargo espflash flash --release --monitor --partition-table ./partition-table.csv --package underfloor-heating
