test:
	cargo test --exclude underfloor-heating --workspace --target x86_64-unknown-linux-gnu

# --partition-table ./partition-table.csv
run:
	cargo espflash flash --release --monitor --package underfloor-heating
