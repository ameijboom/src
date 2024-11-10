# src

Alternative git frontend based on `libgit` written in Rust.

## Introduction

src is an alternative git frontend that leverages the power of `libgit` and is written in Rust.
It provides various git functionalities with an easy-to-use command-line interface and interactive features.

## Features

- Builtin pager support
- Untracked files as first-class citizens
- Support for signing commits using SSH keys
- Interactive mode (currently only for `src add` / `src checkout`)

## Commands

- Add: Stage changes to be committed.
- Feat: Commit a new feature.
- Fix: Commit a bug fix.
- Commit: Record changes to the repository.
- Amend: Modify the most recent commit.
- Push: Update remote refs along with associated objects.
- Fetch: Download objects and refs from another repository.
- Pull: Fetch from and integrate with another repository or a local branch.
- Sync: Synchronize the local repository with the remote repository.
- List: List repository references.
- Diff: Show changes between commits, commit and working tree, etc.
- Stash: Stash the changes in a dirty working directory away.
- Unstash: Apply the changes recorded in a stash to the working directory.
- Branch: Create a new branch
- Checkout: Switch branches

## Installation

To install src, ensure you have Rust installed, then run:

```bash
cargo install --git https://github.com/dmeijboom/src
```

## Contribution

Contributions are welcome! Please submit a pull request or open an issue to discuss your ideas.

## License

This project is licensed under the MIT License.
Feel free to customize it further based on any additional requirements or preferences you have.
