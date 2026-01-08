# GitHub Summary

Tracking open source work can be hard. This tool is intended to summarize PRs, reviews, and commits for a user on repositories of interest over the last month. Each output first lists the merged commits and open PRs for a repo of reference, then optionally attempts a short summary of the work for that repository.

A few disclaimers: Small and locally run LLMs do not produce sufficient quality or accurate results. The summary is best served as a starter template, if it is not complete bogus. Secondly, a developer's worth and productivity cannot be measured with metrics. Number of commits and PRs are not used to generate summaries.

# Getting Started

This tool is mostly a combination of other tools. It uses `octocrab` to retrieve Github data, [`ollama`](https://ollama.com/) to build LLM based summaries locally, and [`just`](https://github.com/casey/just) to simplify CLI commands.

## Install Rust

Install Rust if you don't already have it:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

If you just want to see merged commits and PRs for the quarter, skip to the [install `just`](#install-just) step. 

## Install `ollama`

After you've cloned the repo, you will have to [install ollama](https://ollama.com/download/linux) if you don't already have it. On Linux you can do so with an install script:
```bash
curl -fsSL https://ollama.com/install.sh | sh
```

Some systems will not launch the `ollama` service automatically. Check if it is running with:
```bash
ollama --version
```

If that returns an error, launch the service locally:
```bash
systemctl start ollama.service
```

Now you may download your LLM of choice to your machine. The default for this repo is `llama3.2:3b`:
```bash
ollama run llama3.2:3b
```

After the download and saying hello to the model, close it:
```bash
/bye
```

## Install just

If you do not have `just` on your machine, there are multiple ways to install.

With Cargo:
```bash
cargo install just
```

Or you package manager of choice:
```bash
brew install just
```

For other installation methods, follow the steps [here](https://github.com/casey/just?tab=readme-ov-file#installation).

## Usage

Almost ready to roll. Edit the `config.json` in the repository root. Change `username` to your Github login name. For example, my `username` is `rustaceanrob`. Next, edit the list of tuples of the repository owner names and repositories. For example the `bitcoin` organization holds the `bitcoin` repository.

```json
{
	"username": "rustaceanrob",
	"name": "Robert",
	"description": "Rob contributes to open source software related to Bitcoin and cryptography. He enjoys writing most programs in Rust, but he will also contribute to repositories written with C++, Python, or Swift.",
	"repositories": [
		["bitcoin", "bitcoin"],
		["rust-bitcoin", "rust-bitcoin"],
		["2140-dev", "kyoto"],
		["2140-dev", "swiftsync"],
		["2140-dev", "bitcoin-p2p"],
		["bitcoindevkit", "bdk-ffi"],
		["bitcoindevkit", "bdk-kyoto"],
		["sedited", "kernel-node"],
		["rust-bitcoin", "bip324"]
	],
	"model": "deepseek-r1:14b"
}
```

To fetch all of your merged commits and opened PRs:
```bash
just fetch
```
The summary will be saved to `summary.out`.

To generate an LLM summary for each repository:
```bash
just summarize
```

To watch the result:
```bash
tail -f summary.out
```
