use std::process::ExitStatus;

use crate::JobReturnValue;

#[test]
fn impl_from_unit_works() {
    let value = JobReturnValue::from(());
    assert_eq!(value, JobReturnValue::new(None, true));
}

#[test]
fn impl_from_bool_true_works() {
    let value = JobReturnValue::from(true);
    assert_eq!(value, JobReturnValue::new(None, true));
}

#[test]
fn impl_from_bool_false_works() {
    let value = JobReturnValue::from(false);
    assert_eq!(value, JobReturnValue::new(None, false));
}

#[test]
fn impl_from_str_works() {
    let value = JobReturnValue::from("test");
    assert_eq!(value, JobReturnValue::new(Some("test".into()), true));
}

#[test]
fn impl_from_string_works() {
    let value = JobReturnValue::from("test".to_string());
    assert_eq!(value, JobReturnValue::new(Some("test".into()), true));
}

#[test]
#[cfg(unix)]
fn impl_from_exit_status_success_works() {
    use std::os::unix::process::ExitStatusExt;

    let value = JobReturnValue::from(ExitStatus::from_raw(0 << 8));
    assert_eq!(value, JobReturnValue::new(None, true));
}

#[test]
#[cfg(unix)]
fn impl_from_exit_status_failure_works() {
    use std::os::unix::process::ExitStatusExt;

    let status = ExitStatus::from_raw(1 << 8);
    let value = JobReturnValue::from(status);
    assert_eq!(
        value,
        JobReturnValue::new(Some(status.to_string().into()), false),
    );
}

#[test]
fn impl_from_result_ok_works() {
    let value = JobReturnValue::from(Ok::<_, String>("okyie"));
    assert_eq!(value, JobReturnValue::new(Some("okyie".into()), true));
}

#[test]
fn impl_from_result_err_works() {
    let value = JobReturnValue::from(Err::<(), _>("oopsie"));
    assert_eq!(value, JobReturnValue::new(Some("oopsie".into()), false));
}

#[test]
fn impl_from_option_some_works() {
    let value = JobReturnValue::from(Some("somethin"));
    assert_eq!(value, JobReturnValue::new(Some("somethin".into()), true));
}

#[test]
fn impl_from_option_none_works() {
    let value = JobReturnValue::from(None::<()>);
    assert_eq!(value, JobReturnValue::new(None, false));
}
