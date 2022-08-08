mod k8s;

use k8s::Authenticator;
use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewStatus, UserInfo};
use serde::{Deserialize, Serialize};
use std::error::Error;

register_authenticator!(MyAuth);

#[derive(Serialize, Deserialize)]
pub struct Settings {
    token: String,
    uid: String,
    user: String,
    groups: Vec<String>,
}

struct MyAuth {}
impl k8s::Authenticator<()> for MyAuth {
    fn authenticate(tr: TokenReview, _: ()) -> Result<TokenReview, Box<dyn Error>> {
        Ok(tr)
    }
}

pub struct TestAuthenticator {}

impl k8s::Authenticator<Option<Settings>> for TestAuthenticator {
    fn authenticate(
        tr: TokenReview,
        settings: Option<Settings>,
    ) -> Result<TokenReview, Box<dyn Error>> {
        let token = match tr.spec.token {
            Some(token) => token,
            None => {
                return Err("missing token".into());
            }
        };

        let mut response = TokenReview::default();
        let mut status = TokenReviewStatus::default();

        // get settings or use default values
        let settings = settings.unwrap_or(Settings {
            token: "my-test-token".to_string(),
            uid: "1337".to_string(),
            user: "my-user".to_string(),
            groups: vec!["system:masters".to_string()],
        });

        if token == settings.token {
            status.authenticated = Some(true);
            status.user = Some(UserInfo {
                username: Some(settings.user),
                uid: Some(settings.uid),
                groups: Some(settings.groups),
                extra: None,
            });
        } else {
            status.authenticated = Some(false);
            status.error = Some("invalid token".to_string())
        }

        response.status = Some(status);

        Ok(response)
    }
}

#[test]
fn test1() -> Result<(), Box<dyn Error>> {
    let mut tr = TokenReview::default();
    tr.spec.token = Some("my-test-token".to_string());
    let settings = None;
    let result = TestAuthenticator::authenticate(tr, settings)?;
    let result = serde_json::to_string(&result)?;
    println!("{}", result);
    Ok(())
}

#[test]
fn test3() -> Result<(), Box<dyn Error>> {
    let mut tr = TokenReview::default();
    tr.spec.token = Some("my-test-token1".to_string());
    let settings: Option<Settings> = None;
    let req = k8s::Request::new(tr, settings);
    let input = serde_json::to_vec(&req)?;

    let output = TestAuthenticator::runner().raw_run(&input)?;
    let output = String::from_utf8(output)?;
    println!("{}", output);
    Ok(())
}

#[test]
fn test_no_settings() -> Result<(), Box<dyn Error>> {
    let mut tr = TokenReview::default();
    tr.spec.token = Some("my-test-token1".to_string());
    let req = k8s::Request::new(tr, ());

    let input = serde_json::to_string(&req)?;
    println!("input: '{}'", input);
    let input = input.as_bytes();

    let output = MyAuth::runner().raw_run(&input)?;
    let output = String::from_utf8(output)?;
    println!("output: '{}'", output);
    Ok(())
}