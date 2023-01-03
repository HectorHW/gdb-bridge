# GDB Bridge

This repository contains WIP library for interacting with GDB from Rust.

**NOTE**: currently only supports linux because of ipc built on memfiles and fds inside `/proc`. Feel free to teach me OS basics or submit pull requests if you want to support other systems.

## Installation

First of all, you need to be runnning linux and have gdb installed. After verifying this, add following code to your `Cargo.toml` to the pull library from github:

```toml
[dependencies]
gdb-bridge = {git="https://github.com/HectorHW/gdb-bridge.git"}
```
