# Contributing to Amethyst

Amethyst is an open-source project that values community contribution. We could
always use a helping hand! What would you like to do?

1. [I want to submit issues or request features.](#submitting-issues)
2. [I want to contribute code.](#pull-requests)
3. [I want to write documentation.](#writing-documentation)
4. [Are there any useful resources I can read?] (#useful-resources)

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
   * Unit tests are placed at the bottom of the same .rs file in a submodule
     called `test` with `// Unit tests` right above it. For an example, see the
     unit tests in the [src/engine/timing.rs][ti] file
   * Integration tests are placed in a separate .rs file in the `tests`
     subdirectory.
5. All of the following commands completed without errors.
   * `cargo build`
   * `cargo test`
   * `cargo run` (if it's a binary)

[ml]: https://github.com/ebkalderon/amethyst/blob/master/COPYING
[ti]: https://github.com/ebkalderon/amethyst/blob/master/src/engine/timing.rs#L68-L112

> If you want to be publicly known as an author, feel free to add your name
> and/or GitHub username to the AUTHORS.md file in your pull request.

Once you have pushed your pull request to the repository, please wait for a
reviewer to give feedback on it. If no one responds, feel free to @-reply a
developer or post publicly on [the Gitter chat room][gi] asking for a review.
Once your code has been reviewed, revised if necessary, and then signed-off by a
developer, it will be merged into the source tree.

[gi]: https://gitter.im/ebkalderon/amethyst

Thank you so much for your contribution! Now Amethyst will be a little bit
faster, stronger, and more efficient.

## Writing Documentation

Documentation improvements are always welcome! A solid project needs to have
solid documentation to go with it. You can search for documentation-related
issues on any of our GitHub trackers by filtering by the orange `note: doc`
label.

There are two types of documentation in Amethyst you can work on:

1. [API documentation][ad]
2. [The online Amethyst book][ab]

[ad]: http://ebkalderon.github.io/amethyst/doc/amethyst/
[ab]: http://ebkalderon.github.io/amethyst/

Our Rust API documentation is generated directly from source code comments
marked with either `///` or `//!` using  a tool called Rustdoc. See
[the official Rust book's chapter on Rustdoc][rd] for more information on how
this works.

[rd]: https://doc.rust-lang.org/book/documentation.html

The Amethyst book is generated using a different documentation tool called
[mdBook][mb]. This tool generates pretty HTML e-books from individual Markdown
(.md) files. You can find the source files for this book in the
[book/src/][bk] directory of the Amethyst repository.

[mb]: https://github.com/azerupi/mdBook
[bk]: https://github.com/ebkalderon/amethyst/tree/master/book/src

When submitting your pull requests, please follow the same procedures described
in the [Pull Requests](#pull-requests) section above.

## Useful Resources

* Amethyst
  * [Amethyst Gitter][gi] - The Amethyst project's public chat room.
  * [Development Roadmap][dr] - See this wiki page for general ideas on what you
    can hack on.
* Design Inspiration
  * [Flexible Rendering for Multiple Platforms (2012)][fr]
  * [Mantle Programming Guide and API Reference (2015)][ma]
  * [Misconceptions of Component-Based Entity Systems (2014)][mo]
  * [Nitrous & Mantle: Combining Efficient Engine Design with a Modern API (2014)][ni]
  * [State Pattern - Pushdown Automata (2009)][pa]
* Rust
  * [Rust By Example][re] - Get acquainted with Rust through a series of small
    code samples.
  * [The Rust Programming Language][rl] - The canonical online book about Rust.

[dr]: https://github.com/ebkalderon/amethyst/wiki/Roadmap

[fr]: http://twvideo01.ubm-us.net/o1/vault/gdc2012/slides/Programming%20Track/Persson_Tobias_Flexible_Rendering.pdf.pdf
[ma]: http://www.amd.com/Documents/Mantle-Programming-Guide-and-API-Reference.pdf
[mo]: http://shaneenishry.com/blog/2014/12/27/misconceptions-of-component-based-entity-systems/
[ni]: http://www.gdcvault.com/play/1020706/Nitrous-Mantle-Combining-Efficient-Engine
[pa]: http://gameprogrammingpatterns.com/state.html#pushdown-automata

[re]: http://rustbyexample.com/
[rl]: https://doc.rust-lang.org/book/
