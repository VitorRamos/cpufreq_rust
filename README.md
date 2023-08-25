# cpufreq_lib

Wrapper around cpu filesystem (/sys/devices/system/cpu/) to control various aspects.

## Features

 - Get and set current frenquency and governor.
 - Enable and disable cores.
 - Disable hyperthread

## Crate

 - <https://crates.io/crates/cpufreq_lib>

## Installation

 $ cargo add cpufreq_lib

## Example

```rust
    use cpufreq::CPU;

    let cpu = CPU::new().unwrap();
    let freqs = cpu.online().expect("Unable to read online cpus");
    cpu.disable(1).unwrap();
```