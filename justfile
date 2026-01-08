summarize:
	nohup cargo run --release > summary.out &

fetch:
	nohup cargo run --release noai > summary.out &
