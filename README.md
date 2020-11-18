# A-Team

The tool that helps optimize the code review process.

ATeam gives you two sub-commands: `pr` and `todo`

## ateam pr

This command helps the developers determine which pull request should be reviewed next.

It implemets a ranking system of your open pull requests (excluding the ones with "WIP" label).

Draft pull requiests and pull requires with "WIP" label are excluded.

Pull requests with in progress of failing tests re excluded as well.

It assigns a score to the pull requests. Then, it orders them by score. The highest first, the lowest last.

The ranking algorithm is based on several pieces of information fetched from GitHub.

```
pull request score = 
  (last_commit_age * 10.0) +      # last_commit_age: hours since the last pushed commit
                                  # Older pull requests are shown first.

  ((tests_result-1) * -2000.0) +  # APPLIED BUT, AT THE MOMENT, ALL THE IN PROGRESS OR FAILING PR ARE REMOVED
                                  # tests_result: 0=success, 1=in progress, 2=failing
                                  # - success gives 0
                                  # - in progress subtracts 2000 from the final score
                                  # - failing subtracts 4000 from the final score

  (open_conversations * -20.0) +  # NO MORE AVAILABLE FROM GITHUB, IGNORED
                                  # open_conversations: number of unresolved and non-outdated conversations
                                  # A pull request with open conversations is ranked less than
                                  # one without conversations as you probably better off
                                  # waiting for the conersation to be resolved.

  (approvals^2 * -50.0) +         # approvals: number reviews with state APPROVED
                                  # Approved pull requestes need less attention.

  (reviewers^2 * -20.0) +         # reviewers: number of reviewers
                                  # A pull requestes with many reviewers need less attention.

  (additions * -0.5) +            # additions: number of added lines
                                  # Small pull requests need to be reviewed first.
                                  # They might quickly unblock other pull requests.
                                  # We promote small pull requests.

  (deletions * -0.1)              # deletions: number of deleted lines
                                  # Deleted lines need to be reviewed as well but it
                                  # is usually a quicker job.
```

```
ateam-pr 0.2.0

USAGE:
    ateam pr [FLAGS] [OPTIONS]

FLAGS:
        --exclude-reviewed-by-me       Exclude PRs I have reviewed
    -h, --help                         Prints help information
        --include-mine                 Include my PRs
        --include-tests-failure        Include PRs with tests falure
        --include-tests-in-progress    Include PRs with tests in progess
    -s, --short                        Short version. No table
    -V, --version                      Prints version information

OPTIONS:
        --label <label>...                           Filter by label. Can be used multiple times
    -n, --num <num>                                  Number of pull requests to display
    -q, --query <query>                              GitHub query
        --regex <regex>                              Regexp filter on titles
    -r, --repo <repository>...                       Repositiy. Can be used multiple times to select more than one
        --required-approvals <required-approvals>    Number of required approvals [default: 2]
```

## ateam todo

This second command give you a list of pull requests you are reviewing or you have already reviewed 
that needs your attenction or a list of your pull requests that need your intervention.

Somebody else pull requests:
  - Your approval has been dismissed by a new commit, you need to review again.
  - All your comments has been outdated by new changes. You need to review again.
  - Somebody replied to one of your comment, you need to reply or resolve the conversation.

Your pull requests:
  - Somebody opened a conversation on your pull request. You need to reply or change the code.
  - Somebody asked explicit changes to your pull request.
