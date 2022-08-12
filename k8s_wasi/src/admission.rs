use k8s_openapi::apimachinery::pkg::apis::meta::v1::Status;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;

// we have to define our own AdmissionReview becuase its not included in the
// generated OpenAPI types of Kubernetes and hence not available in the
// k8s_openapi crate. See: https://github.com/kubernetes/kubernetes/issues/84081
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionReview {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<AdmissionRequest>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<AdmissionResponse>,
}

impl Default for AdmissionReview {
    fn default() -> Self {
        Self {
            kind: Some(String::from("AdmissionReview")),
            api_version: Some(String::from("admission.k8s.io/v1")),
            request: Default::default(),
            response: Default::default(),
        }
    }
}

impl AdmissionReview {
    pub fn with_response(response: AdmissionResponse) -> Self {
        let mut ar = Self::default();
        ar.response = Some(response);
        ar
    }

    pub fn accept(uid: String) -> Self {
        Self::with_response(AdmissionResponse::accept(uid))
    }

    pub fn mutate<T: Serialize>(uid: String, mutated_object: T) -> Result<Self, Box<dyn Error>> {
        Ok(Self::with_response(AdmissionResponse::mutate(
            uid,
            mutated_object,
        )?))
    }

    pub fn reject(uid: String) -> Self {
        Self::with_response(AdmissionResponse::reject(uid))
    }

    pub fn reject_with_message(uid: String, message: String) -> Self {
        Self::with_response(AdmissionResponse::reject_with_message(uid, message))
    }

    pub fn get_request(self) -> Result<AdmissionRequest, Box<dyn Error>> {
        match self.request {
            Some(request) => Ok(request),
            None => Err("request not set in AdmissionReview".into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionRequest {
    pub uid: String,
    pub kind: GroupVersionKind,
    pub resource: GroupVersionResource,
    pub sub_resource: Option<String>,
    pub request_kind: Option<GroupVersionKind>,
    pub request_resource: Option<GroupVersionResource>,
    pub request_sub_resource: Option<String>,
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub operation: String,
    pub user_info: k8s_openapi::api::authentication::v1::UserInfo,
    pub object: Option<k8s_openapi::apimachinery::pkg::runtime::RawExtension>,
    pub old_object: Option<k8s_openapi::apimachinery::pkg::runtime::RawExtension>,
    pub dry_run: Option<bool>,
    pub options: Option<k8s_openapi::apimachinery::pkg::runtime::RawExtension>,
}

impl AdmissionRequest {
    pub fn get_object<T: DeserializeOwned>(&mut self) -> Result<T, Box<dyn Error>> {
        let object = std::mem::replace(&mut self.object, None);
        match object {
            None => Err("no object in admission request".into()),
            Some(object) => Ok(serde_json::from_value(object.0)?),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionResponse {
    pub uid: String,
    pub allowed: bool,
    pub status: Option<Status>,
    pub patch: Option<String>,
    pub patch_type: Option<String>,
    pub audit_annotations: Option<BTreeMap<String, String>>,
    pub warnings: Option<Vec<String>>,
}

impl AdmissionResponse {
    pub fn new(uid: String, allowed: bool) -> Self {
        Self {
            uid,
            allowed,
            status: None,
            patch: None,
            patch_type: None,
            audit_annotations: None,
            warnings: None,
        }
    }

    pub fn accept(uid: String) -> Self {
        Self::new(uid, true)
    }

    pub fn reject(uid: String) -> Self {
        Self::new(uid, false)
    }

    pub fn mutate<T: Serialize>(uid: String, mutated_object: T) -> Result<Self, Box<dyn Error>> {
        let mut response = Self::accept(uid);
        response.patch_type = Some("Full".to_string());
        let patch = serde_json::to_vec(&mutated_object)?;
        let patch = base64::encode(patch);
        response.patch = Some(patch);
        Ok(response)
    }

    pub fn reject_with_message(uid: String, message: String) -> Self {
        let mut ar = Self::reject(uid);
        let mut result = Status::default();
        result.message = Some(message);
        ar.status = Some(result);
        ar
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GroupVersionKind {
    pub group: String,
    pub version: String,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GroupVersionResource {
    pub group: String,
    pub version: String,
    pub resource: String,
}
