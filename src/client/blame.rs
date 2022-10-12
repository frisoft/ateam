use futures::stream::{self, StreamExt};
use graphql_client::*;
use rayon::prelude::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/schema.graphql",
    query_path = "src/client/blame.graphql",
    response_derives = "Debug"
)]
pub struct Blame;

pub async fn blame(
    github_api_token: &str,
    repo_name: &str,
    repo_owner: &str,
    files: &Vec<String>,
    login: &str,
) -> bool {
    // println!(">> {:?}", files);
    stream::iter(files)
        .any(|file| async move {
            eprint!(".");

            let response_data: blame::ResponseData =
                match girhub_blame(github_api_token, repo_name, repo_owner, file).await {
                    Ok(data) => data,
                    Err(_) => panic!("Can't get the authors for {}", file),
                };
            is_file_author(&response_data, login)
        })
        .await
}

fn is_file_author(response_data: &blame::ResponseData, login: &str) -> bool {
    // println!("\n\nData: {:?}\n\n", response_data);
    let v = match response_data {
        blame::ResponseData {
            repository:
                Some(blame::BlameRepository {
                    id: _,
                    name: _,
                    default_branch_ref:
                        Some(blame::BlameRepositoryDefaultBranchRef {
                            target:
                                Some(blame::BlameRepositoryDefaultBranchRefTarget::Commit(
                                    blame::BlameRepositoryDefaultBranchRefTargetOnCommit {
                                        blame:
                                            blame::BlameRepositoryDefaultBranchRefTargetOnCommitBlame {
                                                ranges,
                                            },
                                    },
                                )),
                        }),
                }),
        } => Some(ranges),
        _ => None,
    };

    // println!("\n\nRanges: {:?}\n\n", v);

    let authors = match v {
        Some(ranges) => ranges.iter().flat_map(|range|
              match &range.commit.authors.nodes {
                 Some(nodes) => nodes.iter().flat_map(|node|
                     match node {
                        Some(blame::BlameRepositoryDefaultBranchRefTargetOnCommitBlameRangesCommitAuthorsNodes{
                            user: Some(blame::BlameRepositoryDefaultBranchRefTargetOnCommitBlameRangesCommitAuthorsNodesUser {login})
                        }) => Some(login),
                        _ => None
                     }
                 ).collect(),
                 _ => vec!()
              }
        ).collect(),
        _ => vec!(),
    };

    // println!("\n\n Authors: {:?}\n\n", authors);
    authors.contains(&&login.to_string())
}

async fn girhub_blame(
    github_api_token: &str,
    repo_name: &str,
    repo_owner: &str,
    path: &str,
) -> Result<blame::ResponseData, failure::Error> {
    let q = Blame::build_query(blame::Variables {
        repo_name: repo_name.to_string(),
        repo_owner: repo_owner.to_string(),
        path: path.to_string(),
    });

    let res = super::call2(github_api_token, &q).await?;

    // println!(
    // ">>-----------------------------------\n{}\n-------------------------------\n",
    // res.text()?
    // );
    // println!(">> {:?}", res.json()?);
    // println!("{:?}", res);

    let response_body: Response<blame::ResponseData> = res.json().await?;
    // println!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");
        for error in &errors {
            println!("{:?}", error);
        }
    }
    // println!("{:?}", response_body.data);
    Ok(response_body.data.expect("missing response data"))
}
