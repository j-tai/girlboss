use std::process::ExitStatus;

use crate::JobOutput;

#[test]
fn impl_job_output_for_unit_works() {
    let value = ();
    assert_eq!(value.is_success(), true);
    assert_eq!(value.into_message(), None);
}

#[test]
fn impl_job_output_for_bool_true_works() {
    let value = true;
    assert_eq!(value.is_success(), true);
    assert_eq!(value.into_message(), None);
}

#[test]
fn impl_job_output_for_bool_false_works() {
    let value = false;
    assert_eq!(value.is_success(), false);
    assert_eq!(value.into_message(), None);
}

#[test]
fn impl_job_output_for_str_works() {
    let value = "test";
    assert_eq!(value.is_success(), true);
    assert_eq!(value.into_message(), Some("test".into()));
}

#[test]
fn impl_job_output_for_string_works() {
    let value = "test".to_string();
    assert_eq!(value.is_success(), true);
    assert_eq!(value.into_message(), Some("test".into()));
}

#[test]
#[cfg(unix)]
fn impl_job_output_for_exit_status_success_works() {
    use std::os::unix::process::ExitStatusExt;

    let value = ExitStatus::from_raw(0 << 8);
    assert_eq!(value.is_success(), true);
    assert_eq!(value.into_message(), None);
}

#[test]
#[cfg(unix)]
fn impl_job_output_for_exit_status_failure_works() {
    use std::os::unix::process::ExitStatusExt;

    let value = ExitStatus::from_raw(1 << 8);
    assert_eq!(value.is_success(), false);
    assert!(value.into_message().is_some());
}

#[test]
fn impl_job_output_for_result_ok_works() {
    let value: Result<_, String> = Ok("okyie");
    assert_eq!(value.is_success(), true);
    assert_eq!(value.into_message(), Some("okyie".into()));
}

#[test]
fn impl_job_output_for_result_err_works() {
    let value: Result<(), _> = Err("oopsie");
    assert_eq!(value.is_success(), false);
    assert_eq!(value.into_message(), Some("Error: oopsie".into()));
}

#[test]
fn impl_job_output_for_option_some_works() {
    let value = Some("somethin");
    assert_eq!(value.is_success(), true);
    assert_eq!(value.into_message(), Some("somethin".into()));
}

#[test]
fn impl_job_output_for_option_none_works() {
    let value: Option<()> = None;
    assert_eq!(value.is_success(), false);
    assert_eq!(value.into_message(), None);
}
