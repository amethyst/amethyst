# Publishing Amethyst

Publishing a version of Amethyst requires incrementing all modified sub-crates
and then publishing the sub-crates in the following order. You must comment out
dev-dependencies when publishing.

Steps:

- Create a new branch.
- Bump all the versions of all the crates to the new version, push, make sure CI passes.  All crates use the same version as the root amethyst crate for sanity's sake.†
- For non-trivial releases: bump dependencies, push, fix things until CI passes.*
- Review all the PRs since the last release and update the changelog.
  - On the [pull request page], use search filters like: `is:pr is:merged base:master merged:>=YYYY-MM-DD`
  - Reword changelog entries so that they: a) use command form (_Fix..._, not _Fixed..._), and b) describe the problem resolved, not the implementation of the fix -- this often means you have to read the PR to figure it out. For example "Render blue sprites correctly", not "Fixed `render_sprite()` so it uses the correct index into the `rgba` vector".
- Go to [the releases page] and `Draft a new release`.
  - Choose the tag you created earlier.
  - Title should be `Amethyst X.Y.Z` (fill in the version number).
  - Copy in the changelog section from the last step for the body of the post.
- Tag the commit with `vX.Y.Z`, and push.  For example: `git tag v0.15.1 && git push origin HEAD --tags`
  - Comment out the `[dev-dependencies]` section from all `Cargo.toml` files and then commit.  Here is [an example of commenting out `dev-dependencies`].
  - Run `script/publish.sh` from the workspace root.
  - Revert the removal-of-dev-dependencies commit: `git revert HEAD`.
- Get an approval and merge the branch with `bors r+`
- Update the book and API docs on the website.*†
- Bump up the patch version of amethyst_tools in the [tools repository] and publish it with cargo publish.
- Hit publish on your GitHub release entry you drafted earlier.
- Publish a blog post on the website.*
- Announce the release on Discord.
- Post the release to Reddit.*
- Tweet about the release.*
- Update the `amethyst-starter-2d` repo to use the new version

*Could use more detail!

†Let's automate this!

[pull request page]: https://github.com/amethyst/amethyst/pulls
[the releases page]: https://github.com/amethyst/amethyst/releases
[tools repository]: https://github.com/amethyst/tools
[an example of commenting out `dev-dependencies`]: https://github.com/amethyst/amethyst/commit/f911c8b08e960f005fc8013858a971aaa95ac2ed

# Crate dependencies

Here is a snapshot of which crates depend on which others. This is no longer needed for publishing, since the order of publishing is implicit in [publish.sh]

[publish.sh]: https://github.com/amethyst/amethyst/blob/master/script/publish.sh

### Basic Crates

- `amethyst_config`
- `amethyst_derive`
- `amethyst_error`

### Core Crate

- `amethyst_core` <br/> dependencies:
  - Amethyst Error

### Third group

- `amethyst_assets` <br/> dependencies:
  - Amethyst Core
  - Amethyst Derive
  - Amethyst Error
- `amethyst_network` <br/> dependencies:
  - Amethyst Core
  - Amethyst Error
- `amethyst_window` <br/> dependencies:
  - Amethyst Core
  - Amethyst Config
  - Amethyst Error

### Fourth Group

- `amethyst_audio` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Core
  - Amethyst Error
- `amethyst_locale` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Core
  - Amethyst Error
- `amethyst_input` <br/> dependencies:
  - Amethyst Core
  - Amethyst Error
  - Amethyst Config
  - Amethyst Window

### Fifth Group

- `amethyst_controls` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Core
  - Amethyst Error
  - Amethyst Input
- `amethyst_rendy` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Config
  - Amethyst Core
  - Amethyst Derive
  - Amethyst Error
  - Amethyst Window

### Sixth Group

- `amethyst_tiles` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Rendy
  - Amethyst Core
  - Amethyst Error
  - Amethyst Window
- `amethyst_ui` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Audio
  - Amethyst Core
  - Amethyst Derive
  - Amethyst Error
  - Amethyst Input
  - Amethyst Rendy
  - Amethyst Window
- `amethyst_utils` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Controls
  - Amethyst Core
  - Amethyst Error
  - Amethyst Derive
  - Amethyst Rendy
  - Amethyst Window

### Seventh Group

- `amethyst_animation` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Core
  - Amethyst Error
  - Amethyst Derive
  - Amethyst Rendy
  - Amethyst Ui
- `amethyst_gltf` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Animation
  - Amethyst Core
  - Amethyst Error
  - Amethyst Rendy

### Amethyst!

Aka, `amethyst`

Dependencies:
- Amethyst Animation
- Amethyst Assets
- Amethyst Audio
- Amethyst Config
- Amethyst Core
- Amethyst Error
- Amethyst Controls
- Amethyst Derive
- Amethyst Gltf
- Amethyst Network
- Amethyst Locale
- Amethyst Rendy
- Amethyst Input
- Amethyst Ui
- Amethyst Utils
- Amethyst Window

### Post-Amethyst

- `amethyst_test` <br/> dependencies:
  - Amethyst

