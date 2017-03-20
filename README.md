# `wan` ∪･ω･∪
[![Build Status](https://travis-ci.org/ubnt-intrepid/wan.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/wan)
[![Build status](https://ci.appveyor.com/api/projects/status/gn6e5m7plo81fjjl/branch/master?svg=true)](https://ci.appveyor.com/project/ubnt-intrepid/wan/branch/master)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

__Note:__ This project is under development. Some breaking changes will be occured.

## Overview
`wan` is a command-line client of [Wandbox](http://melpon.org/wandbox), written in Rust.
It provides a way to interact with Wandbox from commandline.

## Installation
Rust toolchain is required for installation.
If you don't have installed Rust toolchain yet, visit [official page of rustup](https://www.rustup.rs/) to download installer.

```sh
$ cargo install --git https://github.com/ubnt-intrepid/wan.git
```


## Commands (In progress)

### `wan run <filename> [<filenames>...] [options]`  
Post a code to Wandbox, and retrieve compilation/execution results.  

#### Arguments
- `<filename>`
- `<filenames>...`

#### Options
* `--compiler=<compiler>` - Compiler name  
  By default, the compiler name is automatically detect by extension of `<filename>`.

* `--options=<options>` - Prepared options for used compiler  
  If you want to use multiple options, join them by a comma.

* `--compiler-options=<options>` - Additional options for compiler  
  Arguments are joined by space(s).

* `--runtime-options=<options>`  - Arguments to pass (compiled) executable  
  Arguments are joined by space(s).

#### Example
```cpp
// hello.cpp
#include <iostream>
#include <vector>

int main(int argc, char* argv[]) {
  std::cout << "Hello, wandbox ^ω^" << std::endl;

  std::cout << "Runtime arguments:" << std::endl;
  int i = 0;
  for (auto&& arg: std::vector<char*>(argv, argv+argc)) {
    std::cout << i++ << ": " << arg << std::endl;
  }
  return 0;
}
```

```sh
$ wan run hoge.cpp --compiler=clang-head --runtime-options="a b c"
```

### `wan list`
Get compiler information from Wandbox and list to standard output.

#### Arguments
- none

#### Options
* `--name-only` - Display only compiler name

* `--name <name>` - Filter by compiler name (with regex format)

* `--lang <lang>` - Filter by language (with regex format)

#### Example
```sh
$ wan list
```

### `wan permlink <link>`  
Get a result specified a permlink from Wandbox

#### Arguments
* `<link>` - Permlink

#### Options
* none

#### Example
```sh
$ wan permlink xxxxxxxx
```

## Related Projects
### Wandbox
- [melpon/wandbox](https://github.com/melpon/wandbox)

### Editor Plugin
- [rhysd/wandbox-vim](https://github.com/rhysd/wandbox-vim)
  \- for Vimmers
- [kosh04/emacs-wandbox](https://github.com/kosh04/emacs-wandbox)
  \- for Emacs users
- [wraith13/wandbox-vscode](https://github.com/wraith13/wandbox-vscode)
  \- for VSCode users

### Command Line Client
- [mattn/wandbox-run](https://github.com/mattn/wandbox-run)
  \- Run wandbox as shebang (my inspiration source for developing this project)
- [osyo-manga/gem-wandbox](https://github.com/osyo-manga/gem-wandbox)
  \- written in Ruby
- [rbtnn/go-wandbox](https://github.com/rbtnn/go-wandbox)
  \- written in Golang

### API
- [srz-zumix/wandbox-api](https://github.com/srz-zumix/wandbox-api)
  \- Python
- [Planeshifter/node-wandbox-api](https://github.com/Planeshifter/node-wandbox-api)
  \- Node.js
