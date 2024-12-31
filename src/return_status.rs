use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::Display;
use std::process::ExitStatus;

/// The generalized return value of a [`Job`] function.
///
/// A job function (that is, the function passed to [`Job::start`] or
/// [`Girlboss::start`]) can return any type that implements
/// `Into<JobReturnStatus>`. The return value determines whether or not the job
/// succeeded as well as whether a final status message should be emitted.
///
/// The following types implement `Into<JobReturnStatus>` and hence can be used
/// as the return value of a job function:
///
/// | Type | [`Job::succeeded`] | Final status message ([`Job::status`]) |
/// |:-----|:-------------------|:---------------------------------------|
/// | `()` | `true` | none |
/// | `bool` | the boolean | none |
/// | `&'static str`, `String` | `true` | the string |
/// | [`ExitStatus`] | [`ExitStatus::success`] | none if succeeded, otherwise its `Display` impl |
/// | <code>[Result]&lt;T: Into&lt;JobReturnStatus&gt;, E: [Display]&gt;</code> | [`Result::is_ok`] | the value (if it produces a message) or the error |
/// | <code>[Option]&lt;T: Into&lt;JobReturnStatus&gt;&gt;</code> | [`Option::is_some`] | the value (if it's present and produces a message) |
/// | [`Infallible`] | N/A | N/A |
///
/// [`Job`]: crate::common::Job
/// [`Job::start`]: crate::common::Job::start
/// [`Job::succeeded`]: crate::common::Job::succeeded
/// [`Job::status`]: crate::common::Job::status
/// [`Girlboss::start`]: crate::Girlboss::start
#[derive(Debug, PartialEq, Eq)]
pub struct JobReturnStatus {
    pub(crate) message: Option<Cow<'static, str>>,
    pub(crate) is_success: bool,
}

impl JobReturnStatus {
    pub(crate) fn new(message: Option<Cow<'static, str>>, is_success: bool) -> Self {
        JobReturnStatus {
            message,
            is_success,
        }
    }

    pub(crate) fn panicked() -> Self {
        JobReturnStatus::new(Some("The job panicked".into()), false)
    }
}

impl Default for JobReturnStatus {
    fn default() -> Self {
        Self {
            message: None,
            is_success: true,
        }
    }
}

impl From<()> for JobReturnStatus {
    fn from(_: ()) -> Self {
        JobReturnStatus::default()
    }
}

impl From<bool> for JobReturnStatus {
    fn from(value: bool) -> Self {
        JobReturnStatus::new(None, value)
    }
}

impl From<&'static str> for JobReturnStatus {
    fn from(value: &'static str) -> Self {
        JobReturnStatus::new(Some(value.into()), true)
    }
}

impl From<String> for JobReturnStatus {
    fn from(value: String) -> Self {
        JobReturnStatus::new(Some(value.into()), true)
    }
}

impl From<ExitStatus> for JobReturnStatus {
    fn from(value: ExitStatus) -> Self {
        let is_success = value.success();
        JobReturnStatus {
            message: (!is_success).then(|| value.to_string().into()),
            is_success,
        }
    }
}

impl<T: Into<JobReturnStatus>, E: Display> From<Result<T, E>> for JobReturnStatus {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(value) => value.into(),
            Err(error) => JobReturnStatus {
                message: Some(error.to_string().into()),
                is_success: false,
            },
        }
    }
}

impl<T: Into<JobReturnStatus>> From<Option<T>> for JobReturnStatus {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => inner.into(),
            None => JobReturnStatus::new(None, false),
        }
    }
}

impl From<Infallible> for JobReturnStatus {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
