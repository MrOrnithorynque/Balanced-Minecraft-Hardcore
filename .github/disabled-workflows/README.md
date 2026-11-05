# Disabled workflows

These GitHub Actions workflows are off: GitHub runs only the files in `.github/workflows/`, and this folder
is not that one. They were moved here on 2026-11-05, while GitHub refused every job of the account
("The job was not started because your account is locked due to a billing issue").

- `build.yml` built every jar and ran `jarcheck.py` on each push. The local `verify-loaders` skill covers
  that and more.
- `publish.yml` published a tag to Modrinth, CurseForge, a GitHub release and Discord. The local script
  `.claude/skills/release/publish_local.py` does it now (see the `release` skill).

To turn them back on, on every branch: `mkdir -p .github/workflows && git mv .github/disabled-workflows/build.yml
.github/disabled-workflows/publish.yml .github/workflows/`, then delete this folder. A tag only publishes when
`publish.yml` is in the tagged commit.
