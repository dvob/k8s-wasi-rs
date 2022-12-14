pub mod admission;

pub mod subject_access_review;

pub mod token_review;

use admission::AdmissionReview;
use k8s_openapi::api::{authentication::v1::TokenReview, authorization::v1::SubjectAccessReview};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error,
    fmt::Display,
    io::{Read, Write},
};

#[macro_export]
macro_rules! register_authenticator {
    ($e:ident) => {
        #[no_mangle]
        fn authn() {
            $e::runner().run_with_stdin().unwrap();
        }
    };
}

#[macro_export]
macro_rules! register_authorizer {
    ($e:ident) => {
        #[no_mangle]
        fn authz() {
            $e::runner().run_with_stdin().unwrap();
        }
    };
}

#[macro_export]
macro_rules! register_admiter {
    ($e:ident) => {
        #[no_mangle]
        fn validate() {
            $e::runner().run_with_stdin().unwrap();
        }
    };
}

pub struct RequestRunner<I, O, S> {
    run_fn: fn(input: I, settings: S) -> Result<O, Box<dyn Error>>,
}

impl<I, O, S> RequestRunner<I, O, S>
where
    I: DeserializeOwned,
    S: DeserializeOwned,
    O: Serialize,
{
    pub fn new(f: fn(input: I, settings: S) -> Result<O, Box<dyn Error>>) -> Self {
        Self { run_fn: f }
    }
    pub fn raw_run(&self, input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let response = match serde_json::from_slice::<Request<I, S>>(input) {
            Err(err) => Response {
                response: None,
                error: Some(err.to_string()),
            },
            Ok(request) => (self.run_fn)(request.request, request.settings).into(),
        };
        let output = serde_json::to_vec(&response)?;
        Ok(output)
    }

    pub fn run_with_stdin(&self) -> Result<(), Box<dyn Error>> {
        let mut input = Vec::new();
        std::io::stdin().read_to_end(&mut input)?;

        let output = self.raw_run(input.as_slice())?;

        std::io::stdout().write_all(&output)?;
        std::io::stdout().flush()?;
        Ok(())
    }
}

pub trait Authenticator<S> {
    fn authenticate(tr: TokenReview, settings: S) -> Result<TokenReview, Box<dyn Error>>;

    fn runner() -> RequestRunner<TokenReview, TokenReview, S>
    where
        S: DeserializeOwned,
    {
        RequestRunner::new(Self::authenticate)
    }
}

pub trait Authorizer<S> {
    fn authorize(
        sar: SubjectAccessReview,
        settings: S,
    ) -> Result<SubjectAccessReview, Box<dyn Error>>;

    fn runner() -> RequestRunner<SubjectAccessReview, SubjectAccessReview, S>
    where
        S: DeserializeOwned,
    {
        RequestRunner::new(Self::authorize)
    }
}

pub trait Admiter<S> {
    fn admit(ar: AdmissionReview, settings: S) -> Result<AdmissionReview, Box<dyn Error>>;

    fn runner() -> RequestRunner<AdmissionReview, AdmissionReview, S>
    where
        S: DeserializeOwned,
    {
        RequestRunner::new(Self::admit)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Request<T, S> {
    request: T,
    settings: S,
}

impl<T, S> Request<T, S> {
    pub fn new(request: T, settings: S) -> Self {
        Self { request, settings }
    }
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    response: Option<T>,
    error: Option<String>,
}

impl<T, E> Into<Response<T>> for Result<T, E>
where
    E: Display,
{
    fn into(self) -> Response<T> {
        match self {
            Ok(response) => Response {
                response: Some(response),
                error: None,
            },
            Err(err) => Response {
                response: None,
                error: Some(err.to_string()),
            },
        }
    }
}
