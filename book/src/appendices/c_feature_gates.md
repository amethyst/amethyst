# Appendix C: Feature Gates

Various feature gates exist in Amethyst, with different purposes.
In this chapter, we will go through each of the feature gate types.

## Crate Enabling Feature Gates

To reduce compilation times, you can disable features that are not needed for your project.

When compiling, you can use the following Cargo parameters:

```
cargo (build/test/run) --no-default-features --features feature1,feature2,feature3
```

At the time of writing, the list of features of this type is the following:
* animation
* audio
* gltf
* locale
* network

The full list of available features is available in the Cargo.toml file at the root of the amethyst repository and may change from time to time.

## Profiling

To enable the profiler, you can use the following feature:

```
cargo (build/test/run) --features profiler
```

The next time you will run a project, upon closing it, a file will be created at the root of the project called thread_profile.json.
You can open this file using the chromium browser (or google chrome) and navigating to chrome://tracing

## Nightly

Enabling the nightly feature adds a bit of debug information when running into runtime issues. To use it, you need to use the nightly rust compiler toolchain.

Here is how to enable it:

```
cargo (build/test/run) --features nightly
```

The reason mentionned the most often for using it is to show exactly which resource is missing when a Resources::fetch() fails (or World::read_resource()).
