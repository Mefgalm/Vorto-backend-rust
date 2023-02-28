use std::{
    cell::BorrowError,
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
};

#[derive(Debug)]
pub enum VortoErrorCode {
    Timestamp = 1,
    TeamSize = 2,
    NotFound = 3,
    Validation = 4,
    ActiveGame = 5,
    ExpiredGame = 6,
    TooManyWords = 7,
    InvalidGameToken = 8,
    InvalidLoginOrPassword = 9,
    Infrastructure = 1000,
}

#[derive(Debug, Clone)]
pub struct VortoError {
    pub code: i32,
    pub message: String,
}

impl VortoError {
    pub fn new(error_code: VortoErrorCode, message: String) -> Self {
        Self {
            message: format!("[{:?}] {}", &error_code, message),
            code: error_code as i32,
        }
    }
}

#[derive(Debug)]
pub enum VortoResult<T> {
    Ok(T),
    Err(VortoError),
}

impl<T> VortoResult<T> {
    pub fn expect(&self, message: &'static str) -> &T {
        match self {
            VortoResult::Ok(data) => data,
            VortoResult::Err(_) => std::panic::panic_any(message),
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            VortoResult::Ok(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn unwrap(&self) -> &T {
        match self {
            VortoResult::Ok(data) => data,
            _ => panic!("called `VortoResult::unwrap()` on a `None` value"),
        }
    }
}

impl<T> Try for VortoResult<T> {
    type Output = T;
    type Residual = VortoResult<T>;

    #[inline]
    fn from_output(c: T) -> Self {
        VortoResult::Ok(c)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, T> {
        match self {
            VortoResult::Ok(c) => ControlFlow::Continue(c),
            VortoResult::Err(e) => ControlFlow::Break(VortoResult::Err(e)),
        }
    }
}

impl<T> FromResidual<Result<Infallible, argon2::password_hash::Error>> for VortoResult<T> {
    fn from_residual(x: Result<Infallible, argon2::password_hash::Error>) -> Self {
        match x {
            Err(e) => VortoResult::Err(VortoError::new(
                VortoErrorCode::Infrastructure,
                e.to_string(),
            )),
            Ok(_) => panic!("unreachable"),
        }
    }
}

impl<T> FromResidual<Result<Infallible, sqlx::Error>> for VortoResult<T> {
    fn from_residual(x: Result<Infallible, sqlx::Error>) -> Self {
        match x {
            Err(e) => VortoResult::Err(VortoError::new(
                VortoErrorCode::Infrastructure,
                e.to_string(),
            )),
            Ok(_) => panic!("unreachable"),
        }
    }
}

impl<T> FromResidual<std::option::Option<Infallible>> for VortoResult<T> {
    fn from_residual(x: std::option::Option<Infallible>) -> Self {
        match x {
            None => VortoResult::Err(VortoError::new(
                VortoErrorCode::Infrastructure,
                "Option value is missing".to_owned(),
            )),
            Some(_) => panic!("unreachable"),
        }
    }
}

impl<T> FromResidual<Result<Infallible, reqwest::Error>> for VortoResult<T> {
    fn from_residual(x: Result<Infallible, reqwest::Error>) -> Self {
        match x {
            Err(e) => VortoResult::Err(VortoError::new(
                VortoErrorCode::Infrastructure,
                e.to_string(),
            )),
            Ok(_) => panic!("unreachable"),
        }
    }
}

impl<T> FromResidual<Result<Infallible, BorrowError>> for VortoResult<T> {
    fn from_residual(x: Result<Infallible, BorrowError>) -> Self {
        match x {
            Err(e) => VortoResult::Err(VortoError::new(
                VortoErrorCode::Infrastructure,
                e.to_string(),
            )),
            Ok(_) => panic!("unreachable"),
        }
    }
}

impl<T> FromResidual<Result<Infallible, std::io::Error>> for VortoResult<T> {
    fn from_residual(x: Result<Infallible, std::io::Error>) -> Self {
        match x {
            Err(e) => VortoResult::Err(VortoError::new(
                VortoErrorCode::Infrastructure,
                e.to_string(),
            )),
            Ok(_) => panic!("unreachable"),
        }
    }
}

impl<T> FromResidual<Result<Infallible, jsonwebtokens::error::Error>> for VortoResult<T> {
    fn from_residual(x: Result<Infallible, jsonwebtokens::error::Error>) -> Self {
        match x {
            Err(e) => VortoResult::Err(VortoError::new(
                VortoErrorCode::Infrastructure,
                e.to_string(),
            )),
            Ok(_) => panic!("unreachable"),
        }
    }
}

impl<T, U> FromResidual<VortoResult<U>> for VortoResult<T> {
    fn from_residual(x: VortoResult<U>) -> Self {
        match x {
            VortoResult::Err(e) => VortoResult::Err(From::from(e)),
            VortoResult::Ok(_data) => panic!("unreachable"),
        }
    }
}
