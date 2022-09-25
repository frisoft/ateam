# A-Team

[![Crates.io](https://img.shields.io/crates/v/ateam.svg)](https://crates.io/crates/ateam)
[![Crates.io](https://img.shields.io/crates/d/ateam.svg)](https://crates.io/crates/ateam)
[![CI](https://github.com/frisoft/ateam/workflows/CI/badge.svg)](https://github.com/frisoft/ateam/actions)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/frisoft/ateam/blob/master/LICENSE)

The tool that helps optimize the code review process.

## Install

`cargo install ateam`

Create a GitHub API token and store it in the GITHUB_API_TOKEN env variable. You can also use a .env file.

The token needs `repo` and `read:org` selected scopes.

## Configuration

A-team needs to connect to GitHub's API using your GitHub API token.

You can use this [guide to create one](https://docs.github.com/en/free-pro-team@latest/github/authenticating-to-github/creating-a-personal-access-token#creating-a-token).

The token needs read access to GitHub.

You need to set the token as env variable in your shell. You can add it to your `~.bashrc`, or you can use a `.env` file in the directory you will use ateam from (or one of the parent directories):

```bash
export GITHUB_API_TOKEN=YOUR_TOKEN
```

ATeam gives you two sub-commands: `pr` and `todo`.

## ateam pr

This command helps the developers determine which pull request should be reviewed next.

To get all the pull requests of your organization (any repo), use the following command:

```bash
❯ ateam pr --org OrgName --include-mine --include-reviewed-by-me
```

The previous list also includes your pull requests and all the ones you already reviewed. You probably want to exclude them:

```
❯ ateam pr --org OrgName
```

The pull requests are in the order they are supposed to be reviewed. The first one is probably the one you should review first.

You can also search for specific pull requests. You can use the `--query` option for this. It allows you to use any condition you can use int GitHub search. The most common search is by text:

```
❯ ateam pr --org OrgName --query 'urgent'
```

Unfortunately, the `--query` option does not allow to combine texts with the OR operator.
In the case you want to search for two or more strings, you can use the `--regex` option:

```
❯ ateam pr --org OrgName --regex 'urgent|bugfix|awesome'
```

You can also filter by labels:

```
❯ ateam pr --org OrgName --label LABEL1 --label LABEL2
```

To see all the possible options, you can use `--help`:

```
❯ ateam pr --help

ateam-pr 0.8.5

USAGE:
    ateam pr [OPTIONS]

OPTIONS:
        --batch-size <BATCH_SIZE>
            Mumber of pull requests requested per batch [default: 30]

        --blame
            Look if I changed the same files in the past (SLOW)

    -d, --debug
            Add debug information

        --exclude-label <EXCLUDE_LABEL>
            Exclude pull requests with this label. Can be used multiple times

        --exclude-tests-none
            Exclude pull requests without tests

        --exclude-tests-success
            Exclude pull requests with tests successful

    -h, --help
            Print help information

        --include-drafts
            Include draft pull requests

        --include-mine
            Include my pull requests

        --include-reviewed-by-me
            Include pull requests I have reviewed

        --include-tests-failure
            Include pull requests with tests failure

        --include-tests-pending
            Include pull requests with pending tests

        --json
            Output in JSON

        --label <LABEL>
            Filter by label. Can be used multiple times

    -n, --num <NUM>
            Number of pull requests to display

        --only-mine
            select only my pull requests (enables --include-reviewed-by-me automatically)

        --org <organization>
            Selest all the repositoris of the organization

    -q, --query <QUERY>
            GitHub query. Can be used multiple times

    -r, --repo <repository>
            Repositiy. Can be used multiple times to select more than one

        --regex <REGEX>
            Regexp filter on titles

        --regex-not <REGEX_NOT>
            Regexp filter on titles to exclude pull requests

        --requested
            Select pull requests I have been requested to review, explicitly or as a code owner

        --required-approvals <REQUIRED_APPROVALS>
            Number of required approvals [default: 2]

    -s, --short
            Short version. No table

        --tests-regex <TESTS_REGEX>
            Select tests via regexp. The others are ignored

        --user <USER>
            Query for another user

    -V, --version
            Print version information
```

### How does it work?

It implements a ranking system of your open pull requests.

Draft pull requests are excluded.

Pull requests with pending or failing tests are excluded as well unless you ask for them.

Pull requests you created are excluded too unless you ask for them.

Pull requests you already reviewed are excluded unless you ask for them.

Pull requests with conflicts are excluded.

It assigns a score to the pull requests. Then, it orders them by score. The highest first, the lowest last.

The ranking algorithm is based on several pieces of information:

```
 pull request score =
   last_commit_age * 2.0
   - (tests_result-1) * 2000.0
   - open_conversations * 30.0
   - (approvals - required_approvals) * 80.0
   - (reviews - required_approvals) * 50.0
   - additions * 0.5
   - deletions * 0.1
   + based_on_main_branch * 200.0
   + blame * 400.0
   + requested * 800.0
   + codeowner * 400.0
```

where

`last_commit_age` is the number of hours since the last pushed commit. So, older pull requests will appear first.

`tests_result` is 0 for successful tests, 1 for pending tests and 2 for failing tests. Note that this has only effect if
the --include-tests-failure and/or --include-tests-pending are used.

`open_conversations` is the number of conversation not resolved and not outdated.
A pull request with open conversations is already subject to reviews and discussion and, so, needs less attention.

`approvals` is the number of approvals of the pull requests, and `required_approvals` is the minimum number of approvals required (default = 2).
Approved pull requests need less attention.

`reviews` is the number of reviews the pull request received. A pull requests with many reviews needs less attention.

`additions` is the number of lines added by the pull request. Small pull requests should be reviewed first.
They might quickly unblock other pull requests. We promote small pull requests.

`deletions` is the number of lines removed by the pull request. Small pull requests should be reviewed first.
Deleted lines need to be reviewed as well, but it is usually a quicker job, so they have a lower weight in the formula.

`based_on_main_branch` is 1 if the pull request is based on the main branch. It is 0 if based on another pull request.
It is best reviewing first pull request based on the main branch.

`blame` is 1 if you changed in the past one of the first 5 files changed by the pull requiest.

`requested` is 1 if somebody requested your review explicity, not because you are a code owner.

`codeowner` is 1 if you are one of the [code owners](https://docs.github.com/en/free-pro-team@latest/github/creating-cloning-and-archiving-repositories/about-code-owners) for this pull request.

## ateam followup

This second subcommand gives you some information about the reviews you already submitted and need your attention.

The list of reviews includes:

- Dismissed reviews: A review is usually dismissed when the branch is rebased. You probably want to re-review or re-approve.
- Reviews with addressed conversations: The author replied to your questions or the conversations are outdated
  by the requested changes. The review is in this list only if all your conversations have been addressed.

## ateam todo

NOT AVAILABLE YET

Your pull requests:

- Somebody opened a conversation on your pull request. You need to reply or change the code.
- Somebody asked explicit changes to your pull request.

## Development notes

Ufficial GitHub GraphQL schema: https://docs.github.com/en/graphql/overview/public-schema
