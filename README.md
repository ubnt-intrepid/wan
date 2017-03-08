# wan
[![Build Status](https://travis-ci.org/ubnt-intrepid/wan.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/wan)
[![Build status](https://ci.appveyor.com/api/projects/status/gn6e5m7plo81fjjl/branch/master?svg=true)](https://ci.appveyor.com/project/ubnt-intrepid/wan/branch/master)

わん ∪･ω･∪

`wan` is, a command-line client of Wandbox, written in Rust.

This project is inspired by [mattn/wandbox-run](https://github.com/mattn/wandbox-run).

## Installation

```sh
$ cargo install --git https://github.com/ubnt-intrepid/wan.git
```

## Commands
* `wan list`
* `wan run <compiler> <filename> [<runtime-options>...]`
* `wan-script`

## Usage
* List compiler informations
  ```sh
  $ wan list
  ```

* Post a code to compile & run at Wandbox
  ```sh
  wan run clang-head hoge.cpp a b c
  ```

* Use wandbox as shebang
  ```cpp
  #!/usr/bin/env wan-script

  #include <iostream>

  int main() {
    std::cout << "Hoyaa!" << std::endl;
  }
  ```

  ```sh
  $ chmod +x hoge.cpp
  $ WAN_COMPILER=clang-head ./hoge.cpp
  ```

## License
MIT (See [LICENSE](LICENSE) for details)
