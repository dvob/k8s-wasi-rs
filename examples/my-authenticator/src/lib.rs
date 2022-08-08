use k8s_wasi::Authenticator;
use k8s_openapi::api::authentication::v1::TokenReview;
use k8s_wasi::token_review::*;

struct MyAuthenticator {}

impl Authenticator<()> for MyAuthenticator {
    fn authenticate(tr: TokenReview, _settings: ()) -> Result<TokenReview, Box<dyn std::error::Error>> {
        let _token = get_token(tr)?;

        // verfiy token

        Ok(response_from_status(authenticate(
            "1234".into(),
            "my-user".into(),
            vec![
                "mygroup1".into(),
                "mygroup2".into()
            ])
        ))
    }
}

k8s_wasi::register_authenticator!(MyAuthenticator);