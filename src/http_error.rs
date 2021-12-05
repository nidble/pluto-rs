use std::fmt::Display;

use rweb::hyper::StatusCode;
use rweb::Rejection;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub(crate) struct ErrorMessage {
    pub(crate) code: Option<u16>,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) internal_code: Option<u16>,
}
impl rweb::reject::Reject for ErrorMessage {}

pub(crate) enum HttpError<'a> {
    NotFound(StatusCode),
    BadRequest(StatusCode, &'a ErrorMessage),
    MethodNotAllowed(StatusCode),
    InternalServerError(StatusCode),
}

impl<'a> HttpError<'a> {
    pub(crate) fn resolve_rejection(err: &'a Rejection) -> HttpError<'a> {
        if err.is_not_found() {
            return HttpError::NotFound(StatusCode::NOT_FOUND);
        }

        if let Some(error) = err.find::<ErrorMessage>() {
            return HttpError::BadRequest(StatusCode::BAD_REQUEST, error);
        }

        if err.find::<rweb::reject::MethodNotAllowed>().is_some() {
            return HttpError::MethodNotAllowed(StatusCode::METHOD_NOT_ALLOWED);
        }

        HttpError::InternalServerError(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub(crate) fn format_error<E: Display>(code: u16) -> impl Fn(E) -> Rejection {
    move |err| {
        let error = ErrorMessage {
            internal_code: Some(code),
            message: err.to_string(),
            code: None,
        };
        rweb::reject::custom(error)
    }
}