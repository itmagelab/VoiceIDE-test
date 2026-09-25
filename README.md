# VoiceIDE-test

## `cliff.conf` — example configuration for git-cliff

[`cliff.conf`](cliff.conf) is a fully commented example configuration for
[git-cliff](https://git-cliff.org), the changelog generator that turns
[Conventional Commits](https://www.conventionalcommits.org) history into a
`CHANGELOG.md`.

It shows the three parts of a git-cliff setup:

- **`[changelog]`** — how the document is rendered: header/body/footer Tera
  templates, whitespace trimming, and optional post-processing of the
  finished markdown.
- **`[git]`** — how history is read: conventional-commit parsing, the
  `commit_parsers` mapping from commit type to changelog section, tag
  filtering, and commit ordering.
- **`[remote]`** — optional forge integration (PR links, authors, avatars),
  left commented out. API tokens are never stored in the file — git-cliff
  reads them from `GITHUB_TOKEN`/`GITLAB_TOKEN`/`GITEA_TOKEN`.

With the settings as shipped, the file produces output like:

```markdown
# Changelog

## 1.2.3 - 2026-09-25

### Features
- Add dark mode support

### Bug Fixes
- Fix crash on startup
```

### Using it

git-cliff auto-detects a configuration named `cliff.toml` (or
`~/.config/git-cliff/cliff.toml`), so either point it at this file explicitly:

```console
$ git cliff --config cliff.conf                   # preview on stdout
$ git cliff --config cliff.conf -o CHANGELOG.md   # write CHANGELOG.md
```

or copy it to a name it discovers on its own:

```console
$ cp cliff.conf cliff.toml
```

Every option is optional — git-cliff fills in defaults for anything you
delete, so treat the file as a menu rather than a required schema. For the
full list of settings see the
[git-cliff configuration reference](https://git-cliff.org/docs/configuration).

## Manually running it — GitHub Actions example

[`.github/workflows/manual-changelog.yml`](.github/workflows/manual-changelog.yml)
is an example workflow that is started by hand instead of by a push. Its only
trigger is `workflow_dispatch`, which adds a **Run workflow** button to the
workflow's page in the *Actions* tab and lets it be started from a shell:

```console
$ gh workflow run manual-changelog.yml --ref main
$ gh workflow run manual-changelog.yml --ref main -f range=v1.0.0..HEAD
```

The form behind the button takes three inputs — a commit range (text field),
an "unreleased only" checkbox and a runner image (dropdown) — which the
workflow reads as `${{ inputs.* }}`. It checks out the full history (git-cliff
needs the tags, not just the last commit), installs git-cliff, renders the
changelog with `cliff.conf` and then:

- prints it in the job log,
- appends it to the run's summary page,
- uploads it as a `changelog` artifact.

The result is written nowhere else, so the workflow runs with the narrowest
possible `permissions: contents: read`. Copy the file and swap the steps for
whatever should run on demand — the `on: workflow_dispatch` block stays the
same.
