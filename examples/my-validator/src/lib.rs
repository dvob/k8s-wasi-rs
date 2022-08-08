use k8s_openapi::serde::Deserialize;
use k8s_wasi::Admiter;
use k8s_wasi::admission::AdmissionReview;
use k8s_openapi::api::core::v1::Pod;

#[derive(Deserialize)]
struct Settings {
    pod_name: String,
}

struct MyValidator {}

impl Admiter<Settings> for MyValidator {
    fn admit(ar: AdmissionReview, settings: Settings) -> Result<AdmissionReview, Box<dyn std::error::Error>> {
        let mut request = ar.get_request()?;

        // verify request
        let pod: Pod = request.get_object()?;

        // verfiy pod
        Ok(if pod.metadata.name.unwrap_or_default() == settings.pod_name {
            AdmissionReview::reject_with_message(request.uid, format!("invalid pod name: {}", settings.pod_name))
        } else {
            AdmissionReview::accept(request.uid)
        })
    }
}

k8s_wasi::register_admiter!(MyValidator);