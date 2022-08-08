use k8s_wasi::Authenticator;
use k8s_openapi::api::authentication::v1::TokenReview;

struct MyAuthenticator {}

impl Authenticator<()> for MyAuthenticator {
    fn authenticate(_tr: TokenReview, _settings: ()) -> Result<TokenReview, Box<dyn std::error::Error>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
