use k8s_openapi::api::authorization::v1::SubjectAccessReview;
use k8s_wasi::subject_access_review::*;
use k8s_wasi::Authorizer;

struct MagicAuthorizer {}

impl Authorizer<()> for MagicAuthorizer {
    fn authorize(
        sar: SubjectAccessReview,
        _settings: (),
    ) -> Result<SubjectAccessReview, Box<dyn std::error::Error>> {
        let spec = sar.spec;

        let groups = spec.groups;

        let is_member = groups.map_or(false, |groups| groups.contains(&"magic-group".to_string()));

        if is_member
            && spec.resource_attributes.is_some()
            && spec
                .resource_attributes
                .unwrap_or_default()
                .resource
                .unwrap_or_default()
                == "configmaps"
        {
            Ok(response_from_status(allow()))
        } else {
            Ok(response_from_status(reject()))
        }
    }
}

k8s_wasi::register_authorizer!(MagicAuthorizer);
