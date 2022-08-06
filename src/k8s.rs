use k8s_openapi::api::{authentication::v1::{TokenReview, UserInfo, TokenReviewStatus, TokenReviewSpec}, authorization::v1::SubjectAccessReview};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error,
    io::{Read, Write}, fmt::Display,
};

#[macro_export]
macro_rules! hook_auth {
    ($e:expr) => {
        #[no_mangle]
        fn auth() {
            //let a: $crate::k8s::Authenticator<S> = $e;
            $crate::k8s::Reviewer::new($e).run_with_stdin().unwrap();
        }
    };
}

pub struct Reviewer<I, O> {
    review: fn (input: I) -> Result<O, Box<dyn Error>>
}

impl<I, O> Reviewer<I, O> {
    pub fn new(func: fn(I) -> Result<O, Box<dyn Error>>) -> Self {
        Self {
            review: func,
        }
    }
    pub fn raw_run(&self, input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> 
        where I: DeserializeOwned,
              O: Serialize
    {
        let input = serde_json::from_slice(input)?;
        let resp: Response<O> = (self.review)(input).into();
        let output = serde_json::to_vec(&resp)?;
        Ok(output)
    }

    pub fn run_with_stdin(&self) -> Result<(), Box<dyn Error>>
        where I: DeserializeOwned,
              O: Serialize
    {
        let mut input = Vec::new();
        std::io::stdin().read_to_end(&mut input)?;

        let output = self.raw_run(input.as_slice())?;

        std::io::stdout().write_all(&output)?;
        std::io::stdout().flush()?;
        Ok(())
    }
}

pub type Authenticator<S> = fn (Request<TokenReview, S>) -> Result<TokenReview, Box<dyn Error>>;
pub type Authorizer<S> = fn (Request<SubjectAccessReview, S>) -> Result<SubjectAccessReview, Box<dyn Error>>;
//type Admiter<S> = fn (Request<AR, S>) -> Result<AR, Box<dyn Error>>;

// trait Authenticator<S> {
//     fn authenticate(tr: TokenReview, settings: S) -> Result<TokenReview, Box<dyn Error>>;
// }

//impl<S> Runner<Request<TokenReview, S>, TokenReview> for dyn Authenticator<S> where Self: Sized {
//    fn run(input: Request<TokenReview, S>) -> Result<TokenReview, Box<dyn Error>> {
//        Self::authenticate(input.request, input.settings)
//    }
//}


pub fn authenticate(req: Request<TokenReview, Option<Settings>>) -> Result<TokenReview, Box<dyn Error>> {
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

#[test]
fn test1() -> Result<(), Box<dyn Error>> {
    let mut tr = TokenReview::default();
    tr.spec.token = Some("my-test-token".to_string());
    let settings = None;
    let result = authenticate(Request::new(tr, settings))?;
    let result = serde_json::to_string(&result)?;
    println!("{}", result);
    Ok(())
}

#[test]
fn test2() -> Result<(), Box<dyn Error>> {
    let mut tr = TokenReview::default();
    tr.spec.token = Some("my-test-token1".to_string());
    let settings: Option<Settings> = None;
    let req = Request::new(tr, settings);

    let input = serde_json::to_vec(&req)?;
    let r = Reviewer{ review: authenticate };

    let output = r.raw_run(&input)?;
    let output = String::from_utf8(output)?;
    println!("{}", output);
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    token: String,
    uid: String,
    user: String,
    groups: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Request<T, S> {
    request: T,
    settings: S,
}

impl<T, S> Request<T, S> {
    fn new(request: T, settings: S) -> Self {
        Self {
            request,
            settings,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    response: Option<T>,
    error: Option<String>,
}

impl<T, E> Into<Response<T>> for Result<T, E> where E: Display {
    fn into(self) -> Response<T> {
        match self {
            Ok(response) => Response{
                response: Some(response),
                error: None,
            },
            Err(err) => Response{
                response: None,
                error: Some(err.to_string()),
            },

        }
    }
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

// #[no_mangle]
// fn authn() {
//     stdin_out_wrapper::<TokenReview, TokenReview, Option<Settings>>(my_auth).unwrap();
// }