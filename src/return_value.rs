use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::Display;
use std::process::ExitStatus;

/// The generalized return value of a [`Job`](crate::Job) function.
///
/// A job function (that is, the function passed to [`Job::start`] or
/// [`Girlboss::start`]) can return any type that implements
/// `Into<JobReturnValue>`. The return value determines whether or not the job
/// succeeded as well as whether a final status message should be emitted.
///
/// The following types implement `Into<JobReturnValue>` and hence can be used
/// as the return value of a job function:
///
/// | Type | [`Job::succeeded`] | Final status message ([`Job::status`]) |
/// |:-----|:-------------------|:---------------------------------------|
/// | `()` | `true` | none |
/// | `bool` | the boolean | none |
/// | `&'static str`, `String` | `true` | the string |
/// | [`ExitStatus`] | [`ExitStatus::success`] | none if succeeded, otherwise its `Display` impl |
/// | <code>[Result]&lt;T: Into&lt;JobReturnValue&gt;, E: [Display]&gt;</code> | [`Result::is_ok`] | the value (if it produces a message) or the error |
/// | <code>[Option]&lt;T: Into&lt;JobReturnValue&gt;&gt;</code> | [`Option::is_some`] | the value (if it's present and produces a message) |
/// | [`Infallible`] | N/A | N/A |
///
/// [`Job`]: crate::Job
/// [`Job::start`]: crate::Job::start
/// [`Job::succeeded`]: crate::Job::succeeded
/// [`Job::status`]: crate::Job::status
/// [`Girlboss::start`]: crate::Girlboss::start
#[derive(Debug, PartialEq, Eq)]
pub struct JobReturnValue {
    pub(crate) message: Option<Cow<'static, str>>,
    pub(crate) is_success: bool,
}

impl JobReturnValue {
    pub(crate) fn new(message: Option<Cow<'static, str>>, is_success: bool) -> Self {
        JobReturnValue {
            message,
            is_success,
        }
    }

    pub(crate) fn panicked() -> Self {
        JobReturnValue::new(Some("The job panicked".into()), false)
    }
}

impl Default for JobReturnValue {
    fn default() -> Self {
        Self {
            message: None,
            is_success: true,
        }
    }
}

impl From<()> for JobReturnValue {
    fn from(_: ()) -> Self {
        JobReturnValue::default()
    }
}

impl From<bool> for JobReturnValue {
    fn from(value: bool) -> Self {
        JobReturnValue::new(None, value)
    }
}

impl From<&'static str> for JobReturnValue {
    fn from(value: &'static str) -> Self {
        JobReturnValue::new(Some(value.into()), true)
    }
}

impl From<String> for JobReturnValue {
    fn from(value: String) -> Self {
        JobReturnValue::new(Some(value.into()), true)
    }
}

impl From<ExitStatus> for JobReturnValue {
    fn from(value: ExitStatus) -> Self {
        let is_success = value.success();
        JobReturnValue {
            message: (!is_success).then(|| value.to_string().into()),
            is_success,
        }
    }
}

impl<T: Into<JobReturnValue>, E: Display> From<Result<T, E>> for JobReturnValue {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(value) => value.into(),
            Err(error) => JobReturnValue {
                message: Some(error.to_string().into()),
                is_success: false,
            },
        }
    }
}

impl<T: Into<JobReturnValue>> From<Option<T>> for JobReturnValue {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => inner.into(),
            None => JobReturnValue::new(None, false),
        }
    }
}

impl From<Infallible> for JobReturnValue {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
