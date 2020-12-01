# A-Team

The tool that helps optimize the code review process.

ATeam gives you two sub-commands: `pr` and `todo`

## Install

`cargo install ateam`

Create a GitHub API token and store it in the GITHUB_API_TOKEN env variable. You can also use a .env file.

## Configuration

A-team needs to connect to GitHub's API using your GitHub API token.

You can use this guide to create one: https://docs.github.com/en/free-pro-team@latest/github/authenticating-to-github/creating-a-personal-access-token#creating-a-token

The token needs read access to GitHub.

You need to set the token as env variable in your shell. You can add it to your `~.bashrc/~/.zshrc`, or you can use a `.env` file in the directory you will use ateam from (or one of the parent directories):

```bash
export GITHUB_API_TOKEN=YOUR_TOKEN
```

## ateam pr

This command helps the developers determine which pull request should be reviewed next.

To get all the pull requests of your organization (any repo), use the following command:

```bash
❯ ateam pr --org GitHubOrgName --include-mine --include-reviewed-by-me
```

The previous list also includes your pull requests and all the ones you already reviewed. You probably want to exclude them:
```
❯ ateam pr --org GitHubOrgName
```

The pull requests are in the order they are supposed to be reviewed. The first one is probably the one you should review first.

You can also search for specific pull requests. You can use the `--query` option for this. It allows you to use any condition you can use int GitHub as well. The most common search is by text:

```
❯ ateam pr --org GitHubOrgName --query 'urgent'
```

Unfortunately, the `--query` option does not allow to combine tests with the OR operator.
In the case you want to search for two or more strigs, you can use the `--regex` option:

```
❯ ateam pr --org GitHubOrgName --regex 'urgent|bugfix|awesome'
```
You can also filter by labels:

```
❯ ateam pr --org GitHubOrgName --label LABEL1 --label LABEL2
```

To see all the possible options, you can use `--help`:

```bash
❯ ateam pr --help

ateam-pr 0.3.1

USAGE:
    ateam pr [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug                        Add debug information
    -h, --help                         Prints help information
        --include-mine                 Include my pull requests
        --include-reviewed-by-me       Include pull requests I have reviewed
        --include-tests-failure        Include pull requests with tests falure
        --include-tests-in-progress    Include pull requests with tests in progess
    -s, --short                        Short version. No table
    -V, --version                      Prints version information

OPTIONS:
        --label <label>...                           Filter by label. Can be used multiple times
    -n, --num <num>                                  Number of pull requests to display
        --org <organization>                         Selest all the repositoris of the organization
    -q, --query <query>                              GitHub query
        --regex <regex>                              Regexp filter on titles
    -r, --repo <repository>...                       Repositiy. Can be used multiple times to select more than one
        --required-approvals <required-approvals>    Number of required approvals [default: 2]
```

### How does it work?

It implements a ranking system of your open pull requests.

Draft pull requests are excluded.

Pull requests with in progress or failing tests are excluded as well unless you ask for them.

Pull requests you created are excluded too unless you ask for them.

Pull requests you already reviewed are excluded unless you ask for them.

It assigns a score to the pull requests. Then, it orders them by score. The highest first, the lowest last.

The ranking algorithm is based on several pieces of information:

```
 pull request score = 
   last_commit_age * 10.0
   - (tests_result-1) * 2000.0
   - open_conversations * 30.0
   - (approvals - required_approvals) * 80.0
   - (reviews - required_approvals) * 50.0
   - additions * 0.5
   - deletions * 0.1
```

where

`last_commit_age` is the number of hours since the last pushed commit. So, older pull requests will appear first.

`tests_result` is 0 for successful tests, 1 for in-progress tests and 2 for failing tests. Note that this has only effect if 
the --include-tests-failure and/or --include-tests-in-progress are used.

`open_conversations` is the number of conversation not resolved and not outdated.
A pull request with open conversations is already subject to reviews and discussion and, so, needs less attention. 

`approvals` is the number of approvals of the pull requests, and `required_approvals` is the minimum number of approvals required (default = 2).
Approved pull requests need less attention.

`reviews` is the number of reviews the pull request received. A pull requests with many reviews needs less attention.

`additions` is the number of lines added by the pull request. Small pull requests should be reviewed first.
They might quickly unblock other pull requests. We promote small pull requests.

`deletions` is the number of lines removed by the pull request. Small pull requests should be reviewed first.
Deleted lines need to be reviewed as well, but it is usually a quicker job, so they have a lower weight in the formula.

## ateam todo

NOT AVAILABLE YET

This second command gives you a list of pull requests you are reviewing or you have already reviewed 
that needs your attention or a list of your pull requests that need your intervention.

Somebody else's pull requests:
  - Your approval has been dismissed by a new commit; you need to review again.
  - All your comments have been outdated by recent changes. You need to review it again.
  - Somebody replied to one of your comment. You need to answer or resolve the conversation.

Your pull requests:
  - Somebody opened a conversation on your pull request. You need to reply or change the code.
  - Somebody asked explicit changes to your pull request.


