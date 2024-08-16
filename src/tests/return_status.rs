use std::process::ExitStatus;

use crate::JobReturnStatus;

#[test]
fn impl_from_unit_works() {
    let value = JobReturnStatus::from(());
    assert_eq!(value, JobReturnStatus::new(None, true));
}

#[test]
fn impl_from_bool_true_works() {
    let value = JobReturnStatus::from(true);
    assert_eq!(value, JobReturnStatus::new(None, true));
}

#[test]
fn impl_from_bool_false_works() {
    let value = JobReturnStatus::from(false);
    assert_eq!(value, JobReturnStatus::new(None, false));
}

#[test]
fn impl_from_str_works() {
    let value = JobReturnStatus::from("test");
    assert_eq!(value, JobReturnStatus::new(Some("test".into()), true));
}

#[test]
fn impl_from_string_works() {
    let value = JobReturnStatus::from("test".to_string());
    assert_eq!(value, JobReturnStatus::new(Some("test".into()), true));
}

#[test]
#[cfg(unix)]
fn impl_from_exit_status_success_works() {
    use std::os::unix::process::ExitStatusExt;

    let value = JobReturnStatus::from(ExitStatus::from_raw(0 << 8));
    assert_eq!(value, JobReturnStatus::new(None, true));
}

#[test]
#[cfg(unix)]
fn impl_from_exit_status_failure_works() {
    use std::os::unix::process::ExitStatusExt;

    let status = ExitStatus::from_raw(1 << 8);
    let value = JobReturnStatus::from(status);
    assert_eq!(
        value,
        JobReturnStatus::new(Some(status.to_string().into()), false),
    );
}

#[test]
fn impl_from_result_ok_works() {
    let value = JobReturnStatus::from(Ok::<_, String>("okyie"));
    assert_eq!(value, JobReturnStatus::new(Some("okyie".into()), true));
}

#[test]
fn impl_from_result_err_works() {
    let value = JobReturnStatus::from(Err::<(), _>("oopsie"));
    assert_eq!(value, JobReturnStatus::new(Some("oopsie".into()), false));
}

#[test]
fn impl_from_option_some_works() {
    let value = JobReturnStatus::from(Some("somethin"));
    assert_eq!(value, JobReturnStatus::new(Some("somethin".into()), true));
}

#[test]
fn impl_from_option_none_works() {
    let value = JobReturnStatus::from(None::<()>);
    assert_eq!(value, JobReturnStatus::new(None, false));
}
