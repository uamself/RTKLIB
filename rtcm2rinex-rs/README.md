# RTCM to RINEX Converter (Rust)

A Rust library for converting RTCM (Radio Technical Commission for Maritime Services) format GNSS data to RINEX (Receiver Independent Exchange Format).

This is a Rust reimplementation of RTKLIB's RTCM to RINEX conversion functionality.

## Features

- Convert RTCM2 and RTCM3 messages to RINEX format
- Support for various RINEX versions (2.xx - 3.xx)
- FFI interface for calling from C/C++ code
- Command line tool for easy conversions

## Usage

### Command Line Tool

```bash
# Basic usage
rtcm2rinex -i input.rtcm -o output.rinex

# Specify RINEX version
rtcm2rinex -i input.rtcm -o output.rinex -v 3.04
```

### Library Usage

```rust
use rtcm2rinex::simple_convert;

fn main() {
    // Simple conversion with default options
    match simple_convert("input.rtcm", "output.rinex", 3.04) {
        Ok(_) => println!("Conversion successful"),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Advanced usage with custom options
    let mut ctx = rtcm2rinex::init_rtcm().unwrap();
    
    // Process RTCM file
    rtcm2rinex::process_rtcm_file(&mut ctx, "input.rtcm").unwrap();
    
    // Set up custom RINEX options
    let mut options = rtcm2rinex::RinexOptions::new(3.04);
    
    // Convert to RINEX
    rtcm2rinex::convert_to_rinex(&ctx, &options, "output.rinex").unwrap();
}
```

### C API

The library provides a C-compatible API that can be called from C/C++ programs:

```c
#include <stdio.h>
#include "rtcm2rinex.h"

int main() {
    // Simple conversion
    int result = rtcm2rinex_convert("input.rtcm", "output.rinex", 3.04);
    if (result != 0) {
        fprintf(stderr, "Conversion failed with error code %d\n", result);
        return 1;
    }
    
    // Advanced usage
    void* ctx = rtcm2rinex_init();
    if (ctx == NULL) {
        fprintf(stderr, "Failed to initialize context\n");
        return 1;
    }
    
    // Free resources
    rtcm2rinex_free(ctx);
    
    return 0;
}
```

## Building

```bash
# Build library
cargo build --release

# Run example
cargo run --example simple_convert -- input.rtcm output.rinex 3.04

# Run tests
cargo test
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 