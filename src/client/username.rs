use graphql_client::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/schema.graphql",
    query_path = "src/client/username.graphql",
    response_derives = "Debug"
)]
pub struct Username;

pub fn username(github_api_token: &str) -> String {
    let response_data: username::ResponseData = match github_username(github_api_token) {
        Ok(data) => data,
        Err(_) => panic!("Can't get the username"),
    };

    response_data.viewer.login
}

fn github_username(github_api_token: &str) -> Result<username::ResponseData, failure::Error> {
    let q = Username::build_query(username::Variables {});
    let client = reqwest::Client::new();
    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(github_api_token)
        .json(&q)
        .send()?;

    // println!(
    // ">>-----------------------------------\n{}\n-------------------------------\n",
    // res.text()?
    // );
    // println!(">> {:?}", res.json()?);
    // println!("{:?}", res);

    let response_body: Response<username::ResponseData> = res.json()?;
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
