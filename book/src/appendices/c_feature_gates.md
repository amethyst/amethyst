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

* `animation`
* `audio`
* `gltf`
* `locale`
* `network`
* `renderer`
* `saveload`
* `sdl_controller`

The full list of available features is available in the [Cargo.toml] file.
The available features might change from time to time.

[Cargo.toml]: https://github.com/amethyst/amethyst/blob/master/Cargo.toml

## Graphics features

Whenever you run your game, you'll need to enable one graphics backend. The following features are
available for the backend:

* `empty`
* `metal`
* `vulkan`

Rendy has multiple safety checks built-in to detect bugs in the data it gets submitted. However,
those checks can become too costly for a smooth experience with larger games; you can disable
them using the `no-slow-safety-checks` feature.

Additionally, there's a `shader-compiler` feature which allows compiling GLSL / HLSL to SPIR-V
shaders. This is only needed if you're planning to compile shaders at runtime. Amethyst's 
built-in shaders come pre-compiled, and you can also precompile your own using `glslc` (provided 
by [shaderc]). Please note, that on Windows this feature requires [Ninja] to be installed.

[shaderc]: https://github.com/google/shaderc
[Ninja]: https://ninja-build.org/

## Using Amethyst testing utility

As described in the [Testing chapter][bk_test], Amethyst has several utilities to help you
test an application written using Amethyst. For some cases (especially when rendering components 
are involved in the test), you need to enable the `test-support` feature.

## Profiling

To enable the profiler, you can use the following feature:

```ignore
cargo (build/test/run) --features profiler
```

The next time you will run a project, upon closing it, a file will be created at the root of the project called `thread_profile.json`.
You can open this file using the chromium browser (or google chrome) and navigating to chrome://tracing

## Nightly

> **Note:** Only applicable to Amethyst 0.14 and earlier.
>
> Version after 0.14 no longer have the `"nightly"` feature, as the type names are available on stable Rust.

Enabling the `nightly` feature adds a bit of debug information when running into runtime issues. To
use it, you need to use the nightly rust compiler toolchain.

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

[bk_test]: ../testing.html