use k8s_openapi::api::authentication::v1::{TokenReview, UserInfo, TokenReviewStatus};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error,
    io::{Read, Write},
};

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
struct Request<T, S> {
    request: T,
    settings: S,
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    response: Option<T>,
    error: Option<String>,
}

type RunFn<I, O, S> = fn(req: Request<I, S>) -> Result<O, Box<dyn Error>>;

//type AuthFn<T> = fn(tr: TokenReview, T) -> Result<TokenReview, Box<dyn Error>>;


fn my_auth(req: Request<TokenReview, Option<Settings>>) -> Result<TokenReview, Box<dyn Error>> {
    let tr = req.request;
    let settings = req.settings;
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

fn raw_json_runner<I, O, S>(input: &[u8], run_fn: RunFn<I, O, S>) -> Result<Vec<u8>, Box<dyn Error>>
where
    I: DeserializeOwned,
    S: DeserializeOwned,
    O: Serialize,
{
    let req: Request<I, S> = serde_json::from_slice(input)?;
    let resp = match run_fn(req) {
        Ok(tr) => Response {
            response: Some(tr),
            error: None,
        },
        Err(err) => Response {
            response: None,
            error: Some(err.to_string()),
        },
    };
    let output = serde_json::to_vec(&resp)?;
    Ok(output)
}

fn stdin_out_wrapper<I, O, S>(run_fn: RunFn<I, O, S>) -> Result<(), Box<dyn Error>> 
where
    I: DeserializeOwned,
    O: Serialize,
    S: DeserializeOwned
{
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;

    let output = raw_json_runner(input.as_slice(), run_fn)?;

    std::io::stdout().write_all(&output)?;
    std::io::stdout().flush()?;
    Ok(())
}

#[no_mangle]
fn authn() {
    stdin_out_wrapper::<TokenReview, TokenReview, Option<Settings>>(my_auth).unwrap();
}