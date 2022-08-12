use k8s_openapi::api::authentication::v1::TokenReview;
use k8s_wasi::token_review::*;
use k8s_wasi::Authenticator;

struct MagicAuthenticator {}

impl Authenticator<Option<()>> for MagicAuthenticator {
    fn authenticate(
        tr: TokenReview,
        _settings: Option<()>,
    ) -> Result<TokenReview, Box<dyn std::error::Error>> {
        let token = get_token(tr)?;

        if token == "magic-token" {
            Ok(response_from_status(authenticate(
                "0".into(),
                "magic-user".into(),
                vec!["magic-group".to_string()],
            )))
        } else {
            Ok(response_from_status(reject()))
        }
    }
}

k8s_wasi::register_authenticator!(MagicAuthenticator);
