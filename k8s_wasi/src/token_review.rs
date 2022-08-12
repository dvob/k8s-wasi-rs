use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewStatus, UserInfo};
use std::error::Error;

pub fn get_token(tr: TokenReview) -> Result<String, Box<dyn Error>> {
    match tr.spec.token {
        Some(token) => Ok(token),
        None => Err("no token in request".into()),
    }
}

pub fn response_from_status(status: TokenReviewStatus) -> TokenReview {
    let mut tr = TokenReview::default();
    tr.status = Some(status);
    tr
}

pub fn error(message: String) -> TokenReviewStatus {
    TokenReviewStatus {
        audiences: None,
        authenticated: Some(false),
        user: None,
        error: Some(message),
    }
}

pub fn reject() -> TokenReviewStatus {
    TokenReviewStatus {
        audiences: None,
        authenticated: Some(false),
        user: None,
        error: None,
    }
}

pub fn authenticate(uid: String, username: String, groups: Vec<String>) -> TokenReviewStatus {
    TokenReviewStatus {
        audiences: None,
        authenticated: Some(true),
        user: Some(UserInfo {
            uid: Some(uid),
            username: Some(username),
            groups: Some(groups),
            extra: None,
        }),
        error: None,
    }
}
