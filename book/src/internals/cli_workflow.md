# Command-Line Workflow

## Goals

## Non-Goals

## High-Level Design

## Example Usage

```bash
$ amethyst new mygame
$ cd mygame
$ ls
resources src Cargo.toml
$ ls src
main.rs
$ ls resources
config.yml
$ amethyst add rendering --opengl
$ amethyst add physics --bullet
$ amethyst remove physics
$ amethyst add scripting --ruby
$ ls resources
models scripts shaders textures config.yml renderer.yml
$ amethyst run
[running code]
$
```
