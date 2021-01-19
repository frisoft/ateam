use graphql_client::*;
// use rayon::prelude::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/schema.graphql",
    query_path = "src/client/followup.graphql",
    response_derives = "Debug"
)]
pub struct Followup;

type URI = String;

pub fn followup(
    github_api_token: &str,
    login: &str,
) {
    let response_data: followup::ResponseData =
        match girhub_followup(github_api_token, login) {
            Ok(data) => data,
            Err(_) => panic!("Can't get the follow up actions"),
        };
    
    println!(">> {:?}", &response_data);
    
    parse(&response_data);
    // println!(">> {:?}", files);
}

fn girhub_followup(
    github_api_token: &str,
    login: &str,
) -> Result<followup::ResponseData, failure::Error> {
    let q = Followup::build_query(followup::Variables {
        login: login.to_string(),
    });
    let client = reqwest::Client::new();
    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(github_api_token)
        .json(&q)
        .send()?;

    let response_body: Response<followup::ResponseData> = res.json()?;
    // println!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");
        for error in &errors {
            println!("{:?}", error);
        }
    }
    Ok(response_body.data.expect("missing response data"))
}

fn parse(response_data: &followup::ResponseData) {


}
