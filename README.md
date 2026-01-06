# GitHub Summary

Tracking open source work can be hard. This tool is intended to summarize PRs, reviews, and commits for a user on repositories of interest over the last 3 months (quarter in corpo-speak).

This is a work in progress and does not do anything useful yet.

# Getting Started

Edit the `config.json` in the repository root. Change `username` to your Github login name, _not_ your full name. For example, my `username` is `rustaceanrob`. Next, edit the list of tuples of the repository owner names and repositories. For example the `bitcoin` organization holds the `bitcoin` repository.

```json
{
	"username": "rustaceanrob",
	"repositories": [
		["bitcoin", "bitcoin"],
		["rust-bitcoin", "rust-bitcoin"],
		["2140-dev", "kyoto"],
	]
}
```

You may now fetch metadata on your activity:
```bash
cargo run --release
```
