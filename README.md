## Eppo Multiplatform: SDKs and Artifacts to support Flagging and Experimentation

[![Rust SDK](https://github.com/Eppo-exp/eppo-multiplatform/actions/workflows/ci.yml/badge.svg)](https://github.com/Eppo-exp/eppo-multiplatform/actions/workflows/ci.yml)  
[![Python SDK](https://github.com/Eppo-exp/eppo-multiplatform/actions/workflows/python.yml/badge.svg)](https://github.com/Eppo-exp/eppo-multiplatform/actions/workflows/python.yml)  
[![Ruby SDK](https://github.com/Eppo-exp/eppo-multiplatform/actions/workflows/ruby.yml/badge.svg)](https://github.com/Eppo-exp/eppo-multiplatform/actions/workflows/ruby.yml)  


Eppo is a modular flagging and experimentation analysis tool. Eppo's SDKs are built to make assignments in multi-user server-side and client-side contexts. Before proceeding you'll need an Eppo account.

**Features**
* Feature gates
* Kill switches
* Progressive rollouts
* A/B/n experiments
* Mutually exclusive experiments (Layers)
* Dynamic configuration
* Global holdouts
* Contextual multi-armed bandits

## Contributing

### Preparing your environment

1. Install [rustup](https://rustup.rs/).
2. Install Ruby using your preferred package manager.

### Release process

To release new versions:
1. Look up for a `chore: bump versions before release` pull request, review and merge it.
   - You must publish all bumped packages after merging the PR. If you don't want to publish some of the packages, you may add them to `.changeset/config.json`'s `ignore` field temporarily.
2. If SDK depends on a new version of `eppo_core`, the core should be released first.
   - After a new version of eppo_core is published, Ruby SDK needs its lock file updated (see "Releasing" section in Ruby SDK readme).
2. [Create a new releases](https://github.com/Eppo-exp/rust-sdk/releases/new) for all bumped packages.
   - For tag, use one of the following formats (choose "create new tag on publish"):
     - `eppo_core@x.y.z`
     - `rust-sdk@x.y.z`
     - `python-sdk@x.y.z`
     - `ruby-sdk@x.y.z`
   - Copy release notes from `CHANGELOG.md` file.
   - Publish release.
   - CI will automatically push a new release out to package registries.
