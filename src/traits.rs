use std::borrow::Cow;
use std::fmt::Display;

/// Represents the allowed return types of a job function.
///
/// The return value of a job determines whether or not the job succeeded, as
/// well as whether a final status message should be emitted.
///
/// The following types implement `JobOutput`:
///
/// Type | Is success? | Final status message
/// :----|:------------|:---------------------
/// `()` | yes | none
/// `&'static str` | yes | the string
/// `String` | yes | the string
/// `Result<impl JobOutput, impl Display>` | if the result is `Ok` | the value (if it produces a message) or the error
/// `Option<impl JobOutput>` | if the result is `Some` | the value (if it's present and produces a message)
pub trait JobOutput: Sized {
    /// Returns true if this output represents a success.
    fn is_success(&self) -> bool {
        true
    }

    /// Returns the final status message, if any, represented by this output.
    fn into_message(self) -> Option<Cow<'static, str>> {
        None
    }
}

impl JobOutput for () {}

impl JobOutput for &'static str {
    fn into_message(self) -> Option<Cow<'static, str>> {
        Some(self.into())
    }
}

impl JobOutput for String {
    fn into_message(self) -> Option<Cow<'static, str>> {
        Some(self.into())
    }
}

impl<T: JobOutput, E: Display> JobOutput for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }

    fn into_message(self) -> Option<Cow<'static, str>> {
        match self {
            Ok(value) => value.into_message(),
            Err(error) => Some(format!("Error: {error}").into()),
        }
    }
}

impl<T: JobOutput> JobOutput for Option<T> {
    fn is_success(&self) -> bool {
        self.is_some()
    }

    fn into_message(self) -> Option<Cow<'static, str>> {
        self.and_then(T::into_message)
    }
}
