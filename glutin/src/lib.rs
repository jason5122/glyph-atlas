#![cfg_attr(feature = "cargo-clippy", deny(warnings))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(all(not(egl_backend), not(glx_backend), not(wgl_backend), not(cgl_backend)))]
compile_error!("Please select at least one api backend");

pub mod api;
pub mod config;
pub mod context;
pub mod display;
pub mod error;
pub mod platform;
pub mod prelude;
pub mod surface;

pub(crate) mod private {
    /// Prevent traits from being implemented downstream, since those are used
    /// purely for documentation organization and simplify platform api
    /// implementation maintenance.
    pub trait Sealed {}

    /// `gl_api_dispatch!(match expr; Enum(foo) => foo.something())`
    /// expands to the equivalent of
    /// ```ignore
    /// match self {
    ///    Enum::Egl(foo) => foo.something(),
    ///    Enum::Glx(foo) => foo.something(),
    ///    Enum::Wgl(foo) => foo.something(),
    ///    Enum::Cgl(foo) => foo.something(),
    /// }
    /// ```
    /// The result can be converted to another enum by adding `; as AnotherEnum`
    macro_rules! gl_api_dispatch {
        ($what:ident; $enum:ident ( $($c1:tt)* ) => $x:expr; as $enum2:ident ) => {
            match $what {
                #[cfg(cgl_backend)]
                $enum::Cgl($($c1)*) => $enum2::Cgl($x),
            }
        };
        ($what:ident; $enum:ident ( $($c1:tt)* ) => $x:expr) => {
            match $what {
                #[cfg(cgl_backend)]
                $enum::Cgl($($c1)*) => $x,
            }
        };
    }

    pub(crate) use gl_api_dispatch;
}
