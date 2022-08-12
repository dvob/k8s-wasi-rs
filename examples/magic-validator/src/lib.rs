use k8s_openapi::api::core::v1::ConfigMap;
use k8s_wasi::admission::AdmissionReview;
use k8s_wasi::Admiter;

struct MagicValidator {}

impl Admiter<Option<()>> for MagicValidator {
    fn admit(
        ar: AdmissionReview,
        _settings: Option<()>,
    ) -> Result<AdmissionReview, Box<dyn std::error::Error>> {
        let mut request = ar.get_request()?;

        // verify request
        let config_map: ConfigMap = request.get_object()?;

        let contains_not_allowed_value = config_map
            .data
            .map_or(false, |x| x.contains_key("not-allowed-value"));

        Ok(if contains_not_allowed_value {
            AdmissionReview::reject_with_message(
                request.uid,
                format!("value 'not-allowed-value' not allowed in configmap"),
            )
        } else {
            AdmissionReview::accept(request.uid)
        })
    }
}

k8s_wasi::register_admiter!(MagicValidator);
