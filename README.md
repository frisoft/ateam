# A-Team

The tool that helps optimize the code review process.

ATeam gives you two sub-commands: `pr` and `todo`

## ateam pr

This command helps the developers determine which pull request should be reviewed next.

It implemets a ranking system of your open pull requests.

Draft pull requiests and pull requires with "WIP" label are excluded.

Pull requests with in progress of failing tests are excluded as well, unless you ask for them.

Pull requests you created are excluded as well, unless you ask for them.

Pull requests you alrady reviewed are excluded as well, unless you ask for them.

It assigns a score to the pull requests. Then, it orders them by score. The highest first, the lowest last.

The ranking algorithm is based on several pieces of information:

```
 pull request score = 
   last_commit_age * 10.0 +
   (tests_result-1) * -2000.0 +
   (approvals - required_approvals) * -80.0 +
   (reviews - required_approvals) * -50.0 +
   additions * -0.5 +
   deletions * -0.1
```

where

`last_commit_age` is the number of hours since the last pushed commit. So, older pull requests are shown first.

`tests_result` is 0 for successful tests, 1 for in progress tests and 2 for failing tests. Note that this has only effect if 
the --include-tests-failure and/or --include-tests-in-progress are used.

`approvals` is the number of approvals of the pull requests and `required_approvals` is the minimum number of approcals required (default = 2).
Approved pull requestes need less attention.

`reviews` is the number of reviews the pull requiest received. A pull requestes with many reviews needs less attention.

`additions` is the number of lines added by the pull request. Small pull requests should be reviewed first.
They might quickly unblock other pull requests. We promote small pull requests.

`deletions` is the number of lines removed by the pull request. Small pull requests should be reviewed first.
Deleted lines need to be reviewed as well but it is usually a quicker job, so they have a lower weith in the formula.

```
ateam-pr 0.3.0

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

## ateam todo

NOT AVAILABLE YET

This second command give you a list of pull requests you are reviewing or you have already reviewed 
that needs your attenction or a list of your pull requests that need your intervention.

Somebody else pull requests:
  - Your approval has been dismissed by a new commit, you need to review again.
  - All your comments has been outdated by new changes. You need to review again.
  - Somebody replied to one of your comment, you need to reply or resolve the conversation.

Your pull requests:
  - Somebody opened a conversation on your pull request. You need to reply or change the code.
  - Somebody asked explicit changes to your pull request.

## Install

`cargo install ateam`

Create a GitHub API token and store it in the GITHUB_API_TOKEN env variable. You can also use a .env file.


