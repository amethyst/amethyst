# Contributing to Amethyst

Amethyst is an open-source project that values community contribution. We could
always use a helping hand! What would you like to do?

1. [I want to submit issues or request features.](#submitting-issues)
2. [I want to contribute code or documentation.](#pull-requests)
3. [Are there any useful resources I can read?] (#useful-resources)

## Submitting Issues

One way you can help Amethyst is to report bugs or request features on our
GitHub issue trackers. We can't fix problems we don't know about, so please
report early and often! Make sure to post your issue on the tracker most
relevant to you:

* [Amethyst Tracker][am]: Issues on the core game engine itself or the Amethyst
  Book.
* [Amethyst-CLI Tracker][ac]: Issues on the command-line project management
  tool.

[am]: https://github.com/ebkalderon/amethyst/issues
[ac]: https://github.com/ebkalderon/amethyst_cli/issues

Before posting your issue, please take a moment to search the tracker's existing
issues first, as it's possible that someone else reported the same issue before
you. Though it helps save time, don't worry! We won't mind if you accidentally
post a duplicate issue.

When you are filling out your issue, please label it accordingly. These labels
help developers prioritize what they should work on next. All Amethyst issue
trackers use the same set of labels to mark each issue. Please select one (1)
label from each of the following three sections that is most relevant to your
issue:

1. **Diff**: How difficult do you think it is to resolve?
   * Hard (highest)
   * Medium
   * Easy (least)
2. **Pri**: Relatively speaking, how important is it?
   * Blocker (highest)
   * Critical
   * Important
   * Normal
   * Low (least)
3. **Type**: What kind of issue are you reporting?
   * Bug
     * A problem with the software.
   * Feature
     * New functionality you would like to see.
   * Improvement
     * Making existing functionality better.
   * Question
     * Any kind of question about the project.
   * Task
     * A task or long-term initiative we should pursue.

You may also attach supplementary information using the orange labels beginning
with "Note".

* **Note**: Select any of these if the associated statement is true.
  * Doc
    * This issue deals with documentation.
  * Help Wanted
    * This issue requires lots of people's help to get done.

That's all there is to it! Thanks for posting your issue; we'll take it to heart
and try our best to resolve it.

## Pull Requests

So, you have a pull request? Great! But before you submit, please make sure you
have done the following things first:

1. You have ensured the pull request is against the master branch.
2. You have granted non-exclusive right to your source code under the
   [MIT License][ml].
3. You have processed your source code with `rustfmt`.
4. If your pull request adds new methods or functions to the codebase, you have
   written tests for them.
   * Unit tests are placed in the same .rs file in a submodule called `test` with
     `// Unit tests` right above it.
   * Integration tests are placed in a separate .rs file in the `tests`
     subdirectory.
5. All of the following commands completed without errors.
   * `cargo build`
   * `cargo test`
   * `cargo run` (if it's a binary)

[ml]: https://github.com/ebkalderon/amethyst/blob/master/COPYING

> If you want to be publicly known as an author, feel free to add your name
> and/or GitHub username to the AUTHORS.md file in your pull request.

Once you have pushed your pull request to the repository, please wait for a
reviewer to give feedback on it. If no one responds, feel free to @-reply a
developer or post publicly on [the Gitter chat][gi], asking for a review. Once
your code has been reviewed and signed-off by a developer, it will be merged.

[gi]: https://gitter.im/ebkalderon/amethyst

Thank you so much for your contribution! Now Amethyst will be a little bit
faster, stronger, and more powerful.

## Useful Resources

* Amethyst
  * [Amethyst Gitter][gi] - The Amethyst project's public chat room.
  * [Development Roadmap][dr] - See this wiki page for general ideas on what you
    can hack on.
* Rust
  * [Rust By Example][re] - Get acquainted with Rust through a series of small
    code samples.
  * [The Rust Programming Language][rl] - The canonical online book about Rust.

[dr]: https://github.com/ebkalderon/amethyst/wiki/Roadmap
[re]: http://rustbyexample.com/
[rl]: https://doc.rust-lang.org/book/
