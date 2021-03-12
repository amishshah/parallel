# parallel

[![Rust](https://github.com/amishshah/parallel/actions/workflows/rust.yml/badge.svg)](https://github.com/amishshah/parallel/actions/workflows/rust.yml)

A primitive (but easy-to-use) tool to run multiple system commands in parallel, forwarding on their outputs in order.

## Usage

```bash
$ parallel task1 task2
```

Specify the maximum number of parallel tasks with the `-n` flag (defaults to 4):

```bash
# n=1 is equivalent to running tasks sequentially
# result: 1 2 3
$ parallel -n 1 "echo 1" "echo 2" "echo 3"
```

Run more complex tasks in bash:

```bash
# result: hi bye
$ parallel --shell=bash "sleep 1; echo hi" "echo bye"
```

If every task exits with an error code, then `parallel` will exit with code 1.

## Motivation

I created this tool while learning Rust, don't expect it to be fancy!

Thanks to [RDambrosio016](https://github.com/RDambrosio016) for helping me with questions I had about Rust.
