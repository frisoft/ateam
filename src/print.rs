use super::types::ScoredPr;

pub fn pr(spr: &ScoredPr) {
    let pr = &spr.pr;
    println!("===============================================================");
    println!("PR title: {:?}", pr.title);
    println!("PR URL: {:?}", pr.url);
    println!("Last commit pushed date {:?}", pr.last_commit_pushed_date);
    println!("Tests result {}", pr.tests_result);
    println!("Open conversations {}", pr.open_conversations);
    println!("Approvals {}", pr.num_approvals);
    println!("Reviewers {}", pr.num_reviewers);
    println!("PR additions: {:?}", pr.additions);
    println!("PR deletions: {:?}", pr.deletions);
    println!("Score {:?}", spr.score);
}
