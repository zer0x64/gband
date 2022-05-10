# GBAND
![Logo](logos/gband-2-transparent.png)

## What is GBAND?
GBAND is a GBC emulator with support for async link cable transfer. Link cable might work for some game, but will fail if the rom relies on tight timings or feature some timeout mechanisms.    
[It was original written for NorthSec 2022 CTF](https://github.com/zer0x64/nsec-2022-gband), although CTF-related parts were removed from this repo.

## How to build and run.

### Requirements
The repository uses Git LFS, so you need it to access non-text files in the repo.  
You also need to have a working Rust toolchain set up, and on linux you need `libudev` for gamepad support.  
The emulator uses `wgpu`, which translates graphics call to Vulkan/DirectX12/Metal. Note that only those recent graphic APIs are well supported for now.  

### Emulator
You can refer to the help message for documentation on the different command line arguments.
```
cd gband-wgpu
cargo run --features "gamepad" -- <path/to/rom>
```

### Web Client
This componnent hasn't been cleaned out of the northsec links and references, but still contains a working WebAssembly version of the emulator.
```
cd gband-webclient
trunk serve
```

## License
Code is provided under the MIT or Apache license.
