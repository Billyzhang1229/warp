# Stable-based fork

The maintained fork branch is `stable/dither`, based on upstream
`v0.2026.06.03.09.49.stable_00` (`2249469e5d24e472cee6ce97d3d324293f67db71`).
Only the custom Dither/background-image commits were transplanted. The existing
`feat/dither-background` branch preserves the later development-based version.

`.github/upstream-stable.json` records the exact upstream tag and commit. Do not
use GitHub's **Sync fork** button or merge `upstream/master` into this branch:
those follow development history instead of stable releases.

## Builds

**Fork macOS build** runs on pushes and pull requests targeting `stable/dither`,
and supports manual runs. It uses a public Apple Silicon macOS runner with
Xcode 26, the repository's pinned Rust toolchain and locked Cargo dependencies.
It builds the optimized `warp-oss` binary with the wgpu renderer enabled.

Download `WarpOss-macos-arm64-<commit>` from a successful run's **Artifacts**.
The archive contains `WarpOss-macos-arm64.zip` and its SHA-256 checksum. Unzip the
inner ZIP to obtain `WarpOss.app`. Artifacts expire after 14 days.

The app uses the separate WarpOss application identity. It is ad-hoc signed,
not Apple notarized; macOS may require Open Anyway in Privacy & Security.
`release` is the Cargo optimization profile; `oss` is the application channel;
the upstream **source revision** is the pinned stable release. No private Warp
signing credentials are required. The app includes the source commit, upstream
pin and license attribution; this build omits the optional bundled settings
schema. It does not create a GitHub Release or install updates automatically.

For a local build, install Xcode 26 including MetalToolchain, protobuf,
`cargo-bundle 0.11.0` and `cargo-about 0.8.4`, then run:

```sh
bash script/build-fork-macos
```

## Stable updates

Run **Sync upstream stable** manually in Actions. It selects only published,
non-draft, non-prerelease tags matching `vN.YYYY.MM.DD.HH.MM.stable_NN` across
all release pages. Preview, dev and unrelated releases are excluded even when
GitHub labels them non-prerelease. The selected tag must descend from the pin;
a moved pinned tag is rejected. There is no scheduled synchronization.

A clean merge preserves the custom commits, updates the pin and pushes normally
to `stable/dither`. Conflicts abort without changing the remote branch. A new
merge explicitly calls the build workflow because pushes made with
`GITHUB_TOKEN` do not trigger another push workflow. Check the build result
before using a newly synchronized revision; merges are not automatically
rolled back if compilation later fails.

Both manual workflows must exist on the fork's default branch for GitHub to
show their Run workflow buttons. The intended default is `stable/dither`.

The equivalent local workflow is:

```sh
python3 script/sync-upstream-stable.py          # check only
python3 script/sync-upstream-stable.py --merge  # merge locally, never push
# Resolve compatibility issues, test and build, then:
git push origin HEAD:stable/dither
```

## Validation

The stable port adapts the older settings macro, Rust 2021 syntax and workspace
animation clock without importing the later development API changes. Release
selection and merge behavior have isolated Git-repository tests, including
preserving custom changes, refusing dirty worktrees and aborting conflicts.
Previous development-branch GUI/performance evidence does not validate this
older stable build. Full interactive and performance verification remains
separate from compilation and shader tests.

Stable-port validation passed: 31 distinct Rust tests across the background and
Dither filters, including the explicit image-output test on Apple M4 Pro /
Metal; six Python release-selection/merge tests; WGSL validation; scoped Clippy
with warnings denied; repository Rust formatting; actionlint and shell syntax.
The GPU test covers neutral colors, grayscale, alpha, crop coordinates, masks,
zero-strength Dither and static/animated output. Cloud artifact verification is
recorded by the build Action itself.
