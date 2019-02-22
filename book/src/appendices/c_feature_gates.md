# Appendix C: Feature Gates

Various feature gate exist in Amethyst, with different purposes.
In this chapter, we will go through each of the feature gate types.

## Crate Enabling Feature Gates

To reduce compilation times, you can disable features that are not needed for your project.

When compiling, you can use the following Cargo parameters:

```ignore
cargo (build/test/run) --no-default-features --features feature1,feature2,feature3
```

At the time of writing, the list of features of this type is the following:

* animation
* audio
* gltf
* locale
* network

The full list of available features is available in the [Cargo.toml](https://github.com/amethyst/amethyst/blob/master/Cargo.toml) file.
The available features might change from time to time.

## Profiling

To enable the profiler, you can use the following feature:

```ignore
cargo (build/test/run) --features profiler
```

The next time you will run a project, upon closing it, a file will be created at the root of the project called `thread_profile.json`.
You can open this file using the chromium browser (or google chrome) and navigating to [chrome://tracing](chrome://tracing)

## Nightly

Enabling the nightly feature adds a bit of debug information when running into runtime issues. To use it, you need to use the nightly rust compiler toolchain.

Here is how to enable it:

```ignore
cargo (build/test/run) --features nightly
```

The most common use of this feature is to find out the type name of the resource that is missing, such as when a `Resources::fetch()` or `World::read_resource()` invocation fails.

## Amethyst as a dependency

When using Amethyst as a dependency of your project, you can use the following to disable default features and enable other ones.

```ignore
[dependencies.amethyst]
version = "*"
default-features = false 
features = ["audio", "animation"] # you can add more or replace those
```
