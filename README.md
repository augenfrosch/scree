# scree
Experimental library, directly based on parts of [Physis](https://github.com/redstrate/Physis) and in addition to referencing [ironworks](https://github.com/ackwell/ironworks), that currently only supports checking for the existence of files and folders in FFXIV's SqPack files.

> [!IMPORTANT]
> For most use cases using [Physis](https://github.com/redstrate/Physis) or [ironworks](https://github.com/ackwell/ironworks/tree/main) would be recommended. Due to `scree`'s experimental status it is lacking a ton of features, in addition to stability & testing.

# Building
The [Rust](https://rust-lang.org/) toolchain is required for building. The minimal supported Rust version (MSRV) is `1.95`.

# Examples
There are currently a few examples showcasing the currently available functionality.
Run the following following for a list of them:
```bash
cargo run --example
```
In particular, `verify_current_pathlist` can verify the `CurrentPathList.txt` from [ResLogger2](https://rl2.perchbird.dev/Downloads) in ~5s for an unoptimised build using the default `dev` profile, and ~1s with the default `release` profile. (Barring any bugs...)
