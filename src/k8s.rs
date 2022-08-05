use std::{io::{Write, self}, error::Error};
use k8s_openapi::api::authentication::v1::*;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

trait Authenticator {
   fn authenticate(tr: TokenReview) -> Result<TokenReview, Box<dyn Error>>;
}

#[derive(Serialize, Deserialize)]
struct Settings {
    token: String,
    uid: String,
    user: String,
    groups: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Request<T> {
    request: TokenReview,
    settings: T,
}

#[derive(Serialize, Deserialize)]
struct Response {
    response: Option<TokenReview>,
    error: Option<String>,
}

type AuthFn<T> = fn(tr: TokenReview, T) -> Result<TokenReview, Box<dyn Error>>;

fn my_auth(tr: TokenReview, settings: Option<Settings>) -> Result<TokenReview, Box<dyn Error>> {
    let token = match tr.spec.token {
        Some(token) => token,
        None => {
            return Err("missing token".into());
        }
    };

    let mut response = TokenReview::default();
    let mut status = TokenReviewStatus::default();

    // get settings or use default values
    let settings = settings.unwrap_or(Settings{
        token: "my-test-token".to_string(),
        uid: "1337".to_string(),
        user: "my-user".to_string(),
        groups: vec!["system:masters".to_string()]
    });

    if token == settings.token {
        status.authenticated = Some(true);
        status.user = Some(UserInfo{
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

fn auth_wrapper<T: DeserializeOwned>(input: Box<dyn io::Read>, mut output: Box<dyn io::Write>, auth_fn: AuthFn<T>) -> Result<(), Box<dyn Error>> {
    let req: Request<T> = serde_json::from_reader(input)?;

    let resp = match auth_fn(req.request, req.settings) {
        Ok(tr) => Response{
            response: Some(tr),
            error: None,
        },
        Err(err) => Response{
            response: None,
            error: Some(err.to_string()),
        }
    };


    serde_json::to_writer(&mut output, &resp)?;
    output.flush()?;
    Ok(())
}


#[no_mangle]
fn authn() {
    auth_wrapper::<Option<Settings>>(Box::new(std::io::stdin()), Box::new(std::io::stdout()), my_auth).unwrap();
}