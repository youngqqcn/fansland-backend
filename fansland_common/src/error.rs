//! Errorand Result types.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::Visitor;
use serde::ser::{Serialize, Serializer};
use serde::{Deserialize, Deserializer};
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::io;
use std::sync::PoisonError;

pub type Result<T> = std::result::Result<T, Error>;

/// A generic error that represents all the ways a method can fail inside of rexpr::core.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Default Error
    E(String),
}

impl<T> std::convert::From<PoisonError<T>> for Error {
    fn from(arg: PoisonError<T>) -> Self {
        Error::E(arg.to_string())
    }
}

impl Display for Error {
    // IntellijRust does not understand that [non_exhaustive] applies only for downstream crates
    // noinspection RsMatchCheck
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::E(error) => write!(f, "{}", error),
        }
    }
}

impl StdError for Error {}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Error::from(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(arg: &str) -> Self {
        return Error::E(arg.to_string());
    }
}

impl From<std::string::String> for Error {
    fn from(arg: String) -> Self {
        return Error::E(arg);
    }
}

impl From<&dyn std::error::Error> for Error {
    fn from(arg: &dyn std::error::Error) -> Self {
        return Error::E(arg.to_string());
    }
}

impl From<diesel::result::Error> for Error {
    fn from(arg: diesel::result::Error) -> Self {
        Error::E(arg.to_string())
    }
}

impl From<Error> for std::io::Error {
    fn from(arg: Error) -> Self {
        arg.into()
    }
}

impl From<axum::Error> for Error {
    fn from(arg: axum::Error) -> Self {
        Error::E(arg.to_string())
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Error::from(self.to_string())
    }

    fn clone_from(&mut self, source: &Self) {
        *self = Self::from(source.to_string());
    }
}

// This is what #[derive(Serialize)] would generate.
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

struct ErrorVisitor;

impl<'de> Visitor<'de> for ErrorVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: std::error::Error,
    {
        Ok(v)
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: std::error::Error,
    {
        Ok(v.to_string())
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r = deserializer.deserialize_string(ErrorVisitor)?;
        return Ok(Error::from(r));
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::E(error) => (StatusCode::OK, error).into_response(),
        }
    }
}
