# `insomnia`

[![Documentation](https://docs.rs/insomnia/badge.svg)](https://docs.rs/insomnia)
[![Crates.io](https://img.shields.io/crates/v/insomnia.svg)](https://crates.io/crates/insomnia)
[![License](https://img.shields.io/crates/l/insomnia.svg)](https://github.com/blm768/insomnia/blob/master/LICENSE)

This library provides a cross-platform interface for inhibiting power management operations.

## Features

| Platform                     | Automatic suspend | Manual suspend | Manual shutdown | Screen Sleep |
| ---------------------------- | ----------------- | -------------- | --------------- | ------------ |
| Linux (via `systemd-logind`) | ✓                 | ✓              | ✓               |              |
| Windows                      | ✓                 | ✓              |                 | ✓            |
