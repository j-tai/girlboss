#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod common;
mod error;
mod return_status;
pub mod runtime;
mod status;
mod tests;

pub use error::{Error, Result};
pub use return_status::JobReturnStatus;
pub use status::JobStatus;

#[cfg(not(any(doc, feature = "tokio", feature = "actix-rt")))]
compile_error!("you must specify at least one async runtime as a crate feature");

macro_rules! make_runtime_module {
    ($module:ident = $name:literal , $runtime:ty) => {
        #[doc = concat!("Shortcuts for ", $name, "-specific types.")]
        #[cfg(feature = $name)]
        pub mod $module {
            #[doc = concat!($name, "-specific [`Girlboss`](crate::common::Girlboss) type.")]
            pub type Girlboss<K> = crate::common::Girlboss<$runtime, K>;

            #[doc = concat!($name, "-specific [`Job`](crate::common::Job) type.")]
            pub type Job = crate::common::Job<$runtime>;

            #[doc = concat!($name, "-specific [`Monitor`](crate::common::Monitor) type.")]
            pub type Monitor = crate::common::Monitor<$runtime>;
        }
    };
}

make_runtime_module!(tokio = "tokio", crate::runtime::Tokio);
make_runtime_module!(actix_rt = "actix-rt", crate::runtime::ActixRt);
