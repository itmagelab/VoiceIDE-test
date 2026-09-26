# VoiceIDE-test

## Manually generating a changelog — GitHub Actions example

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
changelog with its built-in defaults and then:

- prints it in the job log,
- appends it to the run's summary page,
- uploads it as a `changelog` artifact.

The result is written nowhere else, so the workflow runs with the narrowest
possible `permissions: contents: read`. Copy the file and swap the steps for
whatever should run on demand — the `on: workflow_dispatch` block stays the
same.
