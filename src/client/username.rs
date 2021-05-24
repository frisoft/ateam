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
        Err(e) => panic!("Can't get the username: {:?}", e),
    };

    response_data.viewer.login
}

fn github_username(github_api_token: &str) -> Result<username::ResponseData, failure::Error> {
    let q = Username::build_query(username::Variables {});
    let res = super::call(github_api_token, &q)?;

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
