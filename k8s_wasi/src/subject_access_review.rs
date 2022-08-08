use k8s_openapi::api::authorization::v1::{SubjectAccessReview, SubjectAccessReviewStatus};

pub fn response_from_status(status: SubjectAccessReviewStatus) -> SubjectAccessReview {
    let mut sar = SubjectAccessReview::default();
    sar.status = Some(status);
    sar
}

pub fn with_reason(mut status: SubjectAccessReviewStatus, reason: String) -> SubjectAccessReviewStatus {
    status.reason = Some(reason);
    status
}

pub fn allow() -> SubjectAccessReviewStatus {
    SubjectAccessReviewStatus{
        allowed: true,
        denied: Some(false),
        evaluation_error: None,
        reason: None,
    }
}

pub fn reject() -> SubjectAccessReviewStatus {
    SubjectAccessReviewStatus{
        allowed: false,
        denied: Some(false),
        evaluation_error: None,
        reason: None,
    }
}