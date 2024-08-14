use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::Display;
use std::process::ExitStatus;

/// Represents the allowed return types of a job function.
///
/// The return value of a job determines whether or not the job succeeded, as
/// well as whether a final status message should be emitted.
///
/// The following types implement `JobOutput`:
///
/// | Type | [`Job::succeeded`](crate::Job::succeeded) | Final status message |
/// |:-----|:------------------------------------------|:---------------------|
/// | `()` | `true` | none |
/// | `bool` | the boolean | none |
/// | `&'static str`, `String` | `true` | the string |
/// | [`ExitStatus`] | [`ExitStatus::success`] | none if succeeded, otherwise its `Display` impl |
/// | <code>[Result]&lt;impl JobOutput, impl Display&gt;</code> | [`Result::is_ok`] | the value (if it produces a message) or the error |
/// | <code>[Option]&lt;impl JobOutput&gt;</code> | [`Option::is_some`] | the value (if it's present and produces a message) |
/// | [`Infallible`] | N/A | N/A |
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

impl JobOutput for bool {
    fn is_success(&self) -> bool {
        *self
    }
}

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

impl JobOutput for ExitStatus {
    fn is_success(&self) -> bool {
        self.success()
    }

    fn into_message(self) -> Option<Cow<'static, str>> {
        if self.success() {
            None
        } else {
            Some(self.to_string().into())
        }
    }
}

impl<T: JobOutput, E: Display> JobOutput for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }

    fn into_message(self) -> Option<Cow<'static, str>> {
        match self {
            Ok(value) => value.into_message(),
            Err(error) => Some(error.to_string().into()),
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

impl JobOutput for Infallible {
    fn is_success(&self) -> bool {
        match *self {}
    }

    fn into_message(self) -> Option<Cow<'static, str>> {
        match self {}
    }
}
