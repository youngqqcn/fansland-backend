use std::borrow::Cow;

use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
};
use fansland_common::RespVO;

// 定义自己的Json extract
pub struct JsonReq<T>(pub T);

// 实现FromRequest
#[async_trait]
impl<S, T> FromRequest<S> for JsonReq<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<RespVO<String>>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(err) => {
                let body: Cow<'_, str> = match err {
                    JsonRejection::JsonDataError(err) => format!("json data error{}", err).into(),
                    JsonRejection::MissingJsonContentType(err) => {
                        format!("must with json format:{}", err).into()
                    }
                    err => format!("error:{}", err).into(),
                };

                Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(RespVO::<String> {
                        code: Some(-1),
                        msg: Some(format!("{}", body)),
                        data: None,
                    }),
                ))
            }
        }
    }
}
