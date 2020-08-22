**Note:** Any interaction with the Amethyst project is subject to our [Code of Conduct](https://github.com/amethyst/amethyst/blob/master/CODE_OF_CONDUCT.md).

# Contributing to Amethyst

Amethyst is an open-source project that values community contribution. We could
always use a helping hand! What would you like to do?

1. [I want to submit issues or request features.](#submitting-issues)
2. [I want to contribute code.](#pull-requests)
3. [I want to write documentation.](#writing-documentation)
4. [Are there any useful resources I can read?](#useful-resources)

## Submitting Issues

One way you can help Amethyst is to report bugs or request features on our
GitHub issue trackers. We can't fix problems we don't know about, so please
report early and often! Make sure to post your issue on the tracker most
relevant to you:

* [Engine Tracker][et]: Issues on the game engine itself or the documentation.
* [Tools Tracker][tt]: Issues on the toolchain surrounding the engine.
* [Website Tracker][wt]: Issues on the website and *This Week in Amethyst*.

[et]: https://github.com/amethyst/amethyst/issues
[tt]: https://github.com/amethyst/tools/issues
[wt]: https://github.com/amethyst/website/issues

Before posting your issue, please take a moment to search the tracker's existing
issues first, as it's possible that someone else reported the same issue before
you. Though it helps save time, don't worry! We won't mind if you accidentally
post a duplicate issue.

Amethyst does not officially support beta or nightly channels of Rust.
If an issue can only be reproduced on those channels our resolution strategy will
be to communicate with the Rust project itself to ensure the issue does not reach
the stable branch.

That's all there is to it! Thanks for posting your issue; we'll take it to heart
and try our best to resolve it.

## Pull Requests

So, you want to write some code? Great!

To begin hacking, make sure you have [git lfs](https://git-lfs.github.com/)
installed, fork the repository to your account and `git clone` the forked
copy to your local machine. On clone you will be on the *master* branch. This
is the branch that contains all new work that has not been released yet. If you
are adding a new feature to amethyst then you want to base your work off of this
branch. Otherwise, if you are attempting to fix something in an older release you
will have to base your work off of a released tag. This can be done after a clone
by running `git checkout TAG` where `TAG` is a released tag eg `v0.8.0`. Now that
you have the latest code you can start contributing.

Contributors should be familiar with the [Git Style Guide](https://github.com/agis/git-style-guide) and [Commit Message Guidelines](https://gist.github.com/robertpainsi/b632364184e70900af4ab688decf6f53).

### Submission Checklist

Before submitting your pull request to the repository, please make sure you have
done the following things first:

1. You have ensured the pull request is based on a recent version of your
   respective branch.
2. You used `cargo fmt` at the root of the crate to format the code.
   Make sure that `cargo fmt --version` returns the latest stable version.
   If this is not the case, run `rustup update` or install [rustfmt]
3. All of the following commands completed without errors.
   * `cargo +stable fmt --all`
   * `cargo clippy --workspace --features "empty"` (may require `cargo clean` before)
   * `cargo build --features "empty"`
   * `cargo test --workspace --features "empty"`
   * `cargo run --example {example-name} --features YOUR_BACKEND`
4. You have granted non-exclusive right to your source code under both the
   [MIT License][lm] and the [Apache License 2.0][la]. Unless you explicitly
   state otherwise, any contribution intentionally submitted for inclusion in
   the work by you, as defined in the Apache 2.0 license, shall be dual
   licensed as above, without any additional terms or conditions.
5. You added your change in docs/CHANGELOG.md and linked your pull request number.
7. For new features or changes to an existing one,
   add or change either the book tutorial or the examples.

[lm]: LICENSE-MIT
[la]: LICENSE-APACHE
[rustfmt]: https://github.com/rust-lang-nursery/rustfmt

Once you have submitted your pull request, please wait for a reviewer to give
feedback on it. If no one responds, feel free to @-mention a developer or post
publicly on [Discord][di] asking for a review. Once
your code has been reviewed, revised if necessary, and then signed-off by a
developer, it will be merged into the source tree.

### Protocol for merging pull requests

The following rules shall be applied strictly for the `amethyst` repository. For other repositories of this organization,
thorough review would be desirable, but no strict application is required there due to lower activity and less influence.

* Pull Requests shall be approved by at least one trusted contributor or member.
* Merging a PR shall be done with `bors r+`, if possible. If there is more than one reviewer the preferred format is `bors r=@reviewer1, @reviewer2`. You can read more about this [here](https://bors.tech/documentation/).
* You may only block merging of a PR if the changes in it are relevant to an org team you have joined. This doesn't mean you can't
make comments, but it does mean that team may ignore your comments if they so choose. Please do not use the "Request Changes" feature if
the PR is not relevant to your team.

Everyone is welcome to review pull requests that they find interesting. It helps save time and improve the code quality for everyone, as well as gaining experience while doing so.

Note: The author of a PR cannot approve their own PR.

##### Special cases

###### Urgent fixes

* If something went wrong (like a broken version has been released, the website doesn't work at all, ..) no approval is required for merging
* Merging can be performed instant (but still with bors)

###### Experimental branches

* If there are very experimental branches, there's no need to use bors; in fact, CI may even fail if that makes working on it easier.
* A review would be good prior to merging, but rules don't need to be strict here

###### Architecture changes, API which influences the workflow / general design of the engine

* Should be labeled with `type: RFC`
* Needs at least three approvals by members only
* Should be left open for reviews for a couple of days

### Dealing With Upstream Changes

When pulling remote changes to a local branch or machine, we recommend users to
rebase instead of creating merge commits.

This is used sometimes when an upstream change cause problems with your pull
request. The best practice is to do a fast-forward ("ff") rebase.

First, setup a remote called `upstream`:

```bash
# Do one of the following. SSH is prefered, but not available on all
# environments.

$ # For ssh
$ git remote add upstream git@github.com:amethyst/amethyst.git
$ # For https
$ git remote add upstream https://github.com/amethyst/amethyst.git
```

If your `origin` remote points to the original repo we recommend you to set it
to your own fork. Check with `git remote origin get-url` to be sure.

```bash
$ # Set origin remote to fork, <your-fork> is git@github.com:<your-username>/amethyst.git
$ # For https use https://github.com/<your-username>/amethyst.git
$ git remote origin set-url <your-fork>
```

To learn how to rebase a upstream change into your branch, please read
[this excellent wiki post][rb].

#### TL;DR

```bash
$ # Fetch latest changes
$ git fetch upstream
$ # Rebase this branch to upstream
$ git checkout <branch-name>
$ git rebase upstream/<branch-to-sync-with>
```

If any errors occur, Git will try to guess what happened. If you can't figure
out how to solve your problem, a quick Google search can help, or you can hit us
up on our [Discord][di] chat.

If needed, abort with `git rebase --abort` and also sometimes
`git merge --abort`.

To check whether anything major has changed upstream, you can do:

```bash
$ # Fetch latest changes
$ git fetch upstream
$ # Do a "non-intrusive" check.
$ git merge --ff-only --no-commit upstream
```

Then you can decide to do a FF rebase. This way, our commit logs remain nice
and clean, and we'll be grateful.

[di]: https://discord.gg/amethyst
[rb]: https://github.com/edx/edx-platform/wiki/How-to-Rebase-a-Pull-Request#how-do-i-rebase

Thank you so much for your contribution! Now Amethyst will be a little bit
faster, stronger, and more efficient.

## Writing Documentation

Documentation improvements are always welcome! A solid project needs to have
solid documentation to go with it. You can search for documentation-related
issues on any of our GitHub trackers by filtering by the green [`projects: docs`][pd]
label.

[pd]: https://github.com/amethyst/amethyst/issues?q=is%3Aopen+is%3Aissue+label%3A"project%3A+docs"

There are two types of documentation in Amethyst you can work on:

1. [API documentation][ad]
2. [The Amethyst book][ab]

[ad]: https://docs.amethyst.rs/master/amethyst/
[ab]: https://book.amethyst.rs/master/

Our Rust API documentation is generated directly from source code comments
marked with either `///` or `//!` using  a tool called Rustdoc. See
[the official Rust book's chapter on Rustdoc][rd] for more information on how
this works.

[rd]: https://doc.rust-lang.org/book/documentation.html

The Amethyst book is generated using a different documentation tool called
[mdBook][mb]. This tool generates pretty HTML e-books from individual Markdown
(.md) files. You can find the source files for this book in the [book/src/][bk]
directory of the Amethyst repository.

[mb]: https://github.com/azerupi/mdBook
[bk]: ../book/src

Documentation of any kind should adhere to the following standard:

1. Lines must not extend beyond 80 characters in length.
2. To enhance readability in text editors and terminals, use only *reference
   style* Markdown links, as shown in the example below. However, if the link
   points to an anchor that exists on the same page, the *inline style* should
   be used instead.
3. Rust code snippets should be compilable whenever reasonably possible.

```markdown
Here is some [example text][et] with a link in it. While we are at it, here is
yet [another link][al]. If we are linking to [an anchor](#anchor) on the same
page, we can do this inline.

[et]: https://some.url/
[al]: https://another.url/
```

Code snippets in markdown files should be surrounded by triple backticks with the modifier `rust,edition2018,no_run,noplaypen`. Use `#` to hide lines that are necessary to compile in doctests but aren't relevant to the example, use `//` for in-code comments.  For example: 

````
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::ecs::{World, WorldExt};

// A simple struct with no data.
struct MyResource;

fn main() {
    // We create a new `World` instance.
    let mut world = World::new();
    
    // We create our resource.
    let my = MyResource;
    
    // We add the resource to the world.
    world.insert(my);
}
```
````

Examples in `book/` can be tested with the following:

```shell
# First, clean up amethyst artifacts and rebuild to ensure there is only one copy of the artifacts in
# the amethyst repo, otherwise mdbook complains. You only need to do this once, unless you change code
# in the actual amethyst library.
rm -rf ./target/debug/deps/libamethyst*
cargo test --workspace --features=empty,tiles --no-run

# Then, test the book.  You can edit, run this command, and then repeat until you get everything passing.
# This is what the book tests in CI do, so the snippets in the book must pass before you can push.
mdbook test -L ./target/debug/deps book
# This serves your book so you can view it locally in a browser and see what it actually looks like
mdbook serve book
```

Examples in the API can be tested with `cargo test`.  Examples in top-level markdown files (like the one we are currently in) are not tested.

When submitting your pull requests, please follow the same procedures described
in the [Pull Requests](#pull-requests) section above, in addition to the following commands:
* `mdbook test book -L target/debug/deps`

Note that if you do not have mdbook already installed, you may do so with `cargo install mdbook`.
If you find dependency resolution problems when testing mdbook,
you may have to run `cargo clean` and `cargo build` again before retrying the `mdbook test` command.

## Profiling the engine
You can build Amethyst with a `profiler` feature like this:

```
cargo build --release --features profiler
```
Or if you wanted to run an example with profiler:
```
cargo run --example my_example --release --features profiler
```
After an Amethyst instance built with `profiler` feature shuts down a
`thread_profile.json` file is generated. It holds information about engine performance
(how much time do various bits of code take to run).
Amethyst uses the same profiling method as [webrender][wr] ([thread_profiler][tp] crate).
`thread_profile.json` can be viewed using Chromium tracing utility.
You can access it by launching Chromium and typing in `about:tracing` in your address bar.
Then you can hit load button and choose `thread_profile.json` file.

## Useful Resources

* Amethyst
  * [Amethyst Discord][di] - The Amethyst project's public chat room.
* Design Inspiration
  * [Bitsquid: Behind The Scenes (2013)][bs]
  * [Flexible Rendering for Multiple Platforms (2012)][fr]
  * [Mantle Programming Guide and API Reference (2015)][ma]
  * [Misconceptions of Component-Based Entity Systems (2014)][mo]
  * [Nitrous & Mantle: Combining Efficient Engine Design with a Modern API (2014)][ni]
  * [State Pattern - Pushdown Automata (2009)][pa]
* Rust
  * [Rust By Example][re] - Get acquainted with Rust through a series of small
    code samples.
  * [The Rust Programming Language][rl] - The canonical online book about Rust.
* Git
  * [How to write a Git commit message](https://chris.beams.io/posts/git-commit/)

[di]: https://discord.gg/amethyst
[bs]: https://www.kth.se/social/upload/5289cb3ff276542440dd668c/bitsquid-behind-the-scenes.pdf
[fr]: http://twvideo01.ubm-us.net/o1/vault/gdc2012/slides/Programming%20Track/Persson_Tobias_Flexible_Rendering.pdf.pdf
[ma]: https://www.yumpu.com/en/document/view/43374261/mantle-programming-guide-and-api-reference
[mo]: https://shanee.io/blog/2014/12/27/misconceptions-of-component-based-entity-systems/
[ni]: http://www.gdcvault.com/play/1020706/Nitrous-Mantle-Combining-Efficient-Engine
[pa]: http://gameprogrammingpatterns.com/state.html#pushdown-automata

[re]: http://rustbyexample.com/
[rl]: https://doc.rust-lang.org/book/
[wr]: https://github.com/servo/webrender/pull/854
[tp]: https://crates.io/crates/thread_profiler
