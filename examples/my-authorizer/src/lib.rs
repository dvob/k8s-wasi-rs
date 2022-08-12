use k8s_openapi::api::authorization::v1::SubjectAccessReview;
use k8s_wasi::subject_access_review::*;
use k8s_wasi::Authorizer;

struct MyAuthorizer {}

impl Authorizer<()> for MyAuthorizer {
    fn authorize(
        sar: SubjectAccessReview,
        _settings: (),
    ) -> Result<SubjectAccessReview, Box<dyn std::error::Error>> {
        let _spec = sar.spec;

        // verfiy spec

        Ok(response_from_status(allow()))
    }
}

k8s_wasi::register_authorizer!(MyAuthorizer);
