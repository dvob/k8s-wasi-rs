use k8s_openapi::api::core::v1::ConfigMap;
use k8s_wasi::admission::AdmissionReview;
use k8s_wasi::Admiter;

struct MagicMutator {}

impl Admiter<()> for MagicMutator {
    fn admit(
        ar: AdmissionReview,
        _settings: (),
    ) -> Result<AdmissionReview, Box<dyn std::error::Error>> {
        let mut request = ar.get_request()?;

        // verify / mutate request
        let mut config_map: ConfigMap = request.get_object()?;

        config_map
            .data
            .get_or_insert_with(Default::default)
            .insert("magic-value".to_string(), "foobar".to_string());

        AdmissionReview::mutate(request.uid, config_map)
    }
}

k8s_wasi::register_admiter!(MagicMutator);
