# acick

[![jobs](https://github.com/gky360/acick/workflows/jobs/badge.svg)](https://github.com/gky360/acick/actions)
[![crates.io](https://img.shields.io/crates/v/acick.svg)](https://crates.io/crates/acick)
[![docs](https://docs.rs/acick/badge.svg)](https://docs.rs/acick)
[![codecov](https://codecov.io/gh/gky360/acick/branch/master/graph/badge.svg)](https://codecov.io/gh/gky360/acick)

```
                           __
                __        /\ \
   __      ___ /\_\    ___\ \ \/'\
 /'__`\   /'___\/\ \  /'___\ \ , <
/\ \L\.\_/\ \__/\ \ \/\ \__/\ \ \\`\
\ \__/.\_\ \____\\ \_\ \____\\ \_\ \_\
 \/__/\/_/\/____/ \/_/\/____/ \/_/\/_/
```

Command line tools for programming contests.

## Features

- Supports some online programming contest services
    - [AtCoder](https://atcoder.jp/)
    - (WIP) [Aizu Online Judge](http://judge.u-aizu.ac.jp/)
- Downloads samples as YAML
- Downloads system testcases
- Compiles and tests your source code with downloaded samples
- Submits your source code

## Requirements

- OS: Linux / OS X / Windows

## Installation

Use [`install.sh`](https://github.com/gky360/acick/blob/master/install.sh) to install binary release.

```
$ curl -sSf -L https://raw.githubusercontent.com/gky360/acick/master/install.sh | sh
```

Or use `cargo` to build from source.

```
$ cargo install acick
```

## Usage

<!-- __ACICK_USAGE_BEGIN__ -->
```
acick 0.1.1-alpha.0

USAGE:
    acick [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -y, --assume-yes    Assumes "yes" as answer to all prompts and run non-interactively
    -h, --help          Prints help information
    -q, --quiet         Hides any messages except the final outcome of commands
    -V, --version       Prints version information

OPTIONS:
    -b, --base-dir <base-dir>    Sets path to the directory that contains a config file
        --output <output>        Specifies the format of output [default: default]  [possible values: default, debug,
                                 json, yaml]

SUBCOMMANDS:
    fetch     Fetches problems from service [aliases: f, f]
    help      Prints this message or the help of the given subcommand(s)
    init      Creates config file
    login     Logs in to service [aliases: l, l]
    logout    Logs out from all services
    me        Gets info of user currently logged in to service
    show      Shows current config
    submit    Submits source code to service [aliases: s, s]
    test      Tests source code with sample inputs and outputs [aliases: t, t]
```
<!-- __ACICK_USAGE_END__ -->

## License

Released under [the MIT license](LICENSE).
