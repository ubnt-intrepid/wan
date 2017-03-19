Vagrantfile/Dockerfile for Local Wandbox Service

## Installation

Create VM and run docker provisioner, in order to build image & run as a container:
```sh
$ vagrant up
```

Restart VM, rsync files and rebuild Docker image:
```sh
$ vagrant reload --provision
```

Run provisioner without rebooting:
```sh
$ vagrant rsync && vagrant provision
```

## Supported Compilers
-  gcc 5.4.0
-  g++ 5.4.0

※ `cattleshed`, `kennel2` のビルドに使用したものを流用しているだけ

## Issues
* ブラウザ上でコードを実行すると `execve: No such file or directory` と表示され終了する
