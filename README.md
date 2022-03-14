## SDL parser library

This library handles parsing SDL input for different programming libraries and environments


- Main parser is written in Rust and offers Rust native library
- `sdl-parser-export` is dynamic library written in Rust, which offers access to parsing logic over Foreign-Function-Interface
- Wrapper libraries offer easy access in mainstream programming libraries, which abstract the logic of calling functions over FFI
  - PyPi library
  - npm library (node)

## Setup

In order to use wrapper libraries, dynamic libraries must be installed on the system.

Once OCR apt repostiory is installed, this can be done
by executing

`sudo apt install libsdl-parser`
