use std::{borrow::Cow, error::Error as StdError};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct ErrWithContext<E: StdError + 'static> {
    context: Cow<'static, str>,
    source: E,
}

impl<E: StdError> std::fmt::Display for ErrWithContext<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context: {}", self.context)
    }
}

pub trait ErrContext {
    type WithContext;
    fn context(self, context: impl Into<Cow<'static, str>>) -> Self::WithContext;
}
impl<T, E: StdError + 'static + Sized> ErrContext for Result<T, E> {
    type WithContext = Result<T, ErrWithContext<E>>;
    fn context(self, context: impl Into<Cow<'static, str>>) -> Self::WithContext {
        self.map_err(|source| ErrWithContext {
            source,
            context: context.into(),
        })
    }
}
