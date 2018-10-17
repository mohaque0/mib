# Modular Incremental Builder

A simple builder for compiling modular C++ projects.

## Motivation

Mib combines different solutions to build problems and offers (eventually) a plugin architecture for integrating more projects to seamlessly work together.

The current implementation uses Conan.io for dependency management and CMake for builds. Conan.io has many more packages available in its repositories than its competitors and CMake is a common cross-platform build generator so it should be available on many systems.

Mib should also make it easy to configure and get started with a modular codebase. Conan.io and CMake both currently require complex scripts, often very similar between modules, for each module in a project. This duplication is unnecessary.

## Building Mib

Mib is written in [Rust](https://www.rust-lang.org/en-US/) and built using Rust's default [Cargo](https://doc.rust-lang.org/cargo/index.html) build system. To build Mib in the root of the source tree run:

```
cargo build
```

To install it as a local repostiory run
```
cd main
cargo install
```

## Using Mib

To get help run:
```
mib --help
```

To get help on a particular plugin ("conan" is the only plugin currently implemented) run
```
mib --help conan
```

### Configuration

Mib configuration is specified in a [YAML](http://yaml.org/) file placed at the root of your project's source tree called "build.yml".
The file has two sections:
* "default" which contains default configuration shared between all modules in your project.
* "modules" which contains a list of module-specific configuration. Each module has:
  * (required) "name" which is the name of the module.
  * (optional) "path" which is the path to the module relative to the folder containing build.yml. By default it assumes the module is in a folder of the same name as the module itself.
  * "config" which is a map of configuration for plugins to use.

Each module places source files in a subfolder called "src" and test files in a subfolder called "test". [Catch2](https://github.com/catchorg/Catch2) is currently the only supported unit testing framework.

### Building with Mib

In the project root run:
```
mib
```

## TODO

Immediate work still to do:
* Conan dependencies should be automatically added based on module dependencies.
* An api for adding plugins should be implemented. The plan that plugins will be executables/scripts that communicate using jsonrpc.
