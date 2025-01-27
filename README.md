# logreduce extract anomaly from log files

Based on success logs, logreduce highlights useful text in failed logs.
The goal is to save time in finding a failure's root cause.

## How it works

logreduce uses a *model* to learn successful logs and detect novelties in
failed logs:

* Random words are manually removed using regular expression,
* Then lines are converted to a matrix of token occurrences
  (using **HashingVectorizer**),
* An unsupervised learner implements neighbor searches
  (using **NearestNeighbors**).


### Caveats

This method doesn't work when debug content is only included in failed logs.
To successfully detect anomalies, failed and success logs needs to be similar,
otherwise the extra informations in failed logs will be considered anomalous.

For example this happens with testr where success logs only contains 'SUCCESS'.


## Install

Install the `logreduce` command line by running:

```
cargo install --git https://github.com/logreduce/logreduce logreduce-cli
```

> If you don't have `cargo`, see this [install rust](https://www.rust-lang.org/tools/install) documentation.

Or grab the latest release assets `logreduce-x86_64-linux.tar.bz2` from https://github.com/logreduce/logreduce/releases


## Use

Analyze a local file:

```ShellSession
$ logreduce file /var/log/zuul/scheduler.log
```

Analyze a remote url:

```ShellSession
$ logreduce url https://zuul/build/uuid
```

Compare two inputs (when baseline discovery doesn't work):

```ShellSession
$ logreduce diff https://zuul/build/success-build https://zuul/build/failed-build
```

Save and re-use trained model using the `--model file-path` argument.


## Configure

Logreduce supports the [ant's fileset](https://ant.apache.org/manual/Types/fileset.html) configuration to
filter the processed files:

- *includes*: list of files regex that must be included. Defaults to all files.
- *excludes*: list of files regex that must be excluded. Defaults to default excludes or none if `default_excludes` is false.
- *default_excludes*: indicates whether [default excludes](./crates/model/src/config/default_excludes.rs) should be used or not.


## Learn

To read more about the project:

- Initial presentation [blog post](https://opensource.com/article/18/9/quiet-log-noise-python-and-machine-learning)
- The command line specification: [./doc/adr/0001-architecture-cli.md](./doc/adr/0001-architecture-cli.md)
- How the tokenizer works: [Improving logreduce tokenizer](https://www.softwarefactory-project.io/improving-logreduce-with-rust.html)
- How the nearest neighbor works: [Implementing logreduce nearest neighbors](https://www.softwarefactory-project.io/implementing-logreduce-nearest-neighbors-model-in-rust.html)
- How the log file iterator works: [Introducing the BytesLines iterator](https://www.softwarefactory-project.io/introducing-the-byteslines-iterator.html)
- [Completing the first release of logreduce-rust](https://www.softwarefactory-project.io/completing-the-first-release-of-logreduce-rust.html)
- How the web interface works: [WASM based web interface](https://www.softwarefactory-project.io/logreduce-wasm-based-web-interface.html)


## Contribute

Clone the project and run tests:

```
git clone https://github.comm/logreduce/logreduce && cd logreduce
cargo test && cargo fmt && cargo clippy
```

Run the project:

```
cargo run -p logreduce-cli -- --help
```

Join the project Matrix room: [#logeduce:matrix.org](https://matrix.to/#/#logreduce:matrix.org).

## Roadmap

* detect `jenkins` url
* Reports minification


[logreduce]: https://github.com/logreduce/logreduce
