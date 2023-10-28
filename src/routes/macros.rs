/// Macro to easily create routes with many path parameters
macro_rules! route_7_levels {
    ($app:ident, $prefix:literal $route:literal, $($method:ident($handler1:expr, $handler2:expr, $handler3:expr, $handler4:expr, $handler5:expr, $handler6:expr, $handler7:expr)),*) => {
        $app = $app
            .route(
                concat!($prefix, "/:one", $route),
                $($method($handler1)).*
            )
            .route(
                concat!($prefix, "/:one/:two", $route),
                $($method($handler2)).*
            )
            .route(
                concat!($prefix, "/:one/:two/:three", $route),
                $($method($handler3)).*,
            )
            .route(
                concat!($prefix, "/:one/:two/:three/:four", $route),
                $($method($handler4)).*,
            )
            .route(
                concat!($prefix, "/:one/:two/:three/:four/:five", $route),
                $($method($handler5)).*,
            )
            .route(
                concat!($prefix, "/:one/:two/:three/:four/:five/:six", $route),
                $($method($handler6)).*,
            )
            .route(
                concat!($prefix, "/:one/:two/:three/:four/:five/:six/:seven", $route),
                $($method($handler7)).*,
            )
            ;
    };
}

/// Macro to replace a captured TokenTree by a Type
macro_rules! replace_expr {
    ($_t:tt -> $sub:tt) => {
        $sub
    };
}

/// Macro to quickly write functions for image names several layers depp
/// eg: `nvidia/dcgm/whatever`
macro_rules! endpoint_fn_7_levels {
    ($fn_name:ident, $($arg:ident: $t:ty),+; image_name/$($p:ident),*; $($pa:ident: $pt:ty),*; -> $ret:ty) => {
        paste::item! {
            pub async fn [< $fn_name _2level >](
                $($arg: $t),*,
                Path((one, two, $($p),*)): Path<(
                    String,
                    String,
                    $($crate::routes::macros::replace_expr!(($p) -> String)),*
                )>,
                $($pa: $pt),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}") $(,$p)*)),
                    $($pa),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _3level >](
                $($arg: $t),*,
                Path((one, two, three, $($p),*)): Path<(
                    String,
                    String,
                    String,
                    $($crate::routes::macros::replace_expr!(($p) -> String)),*
                )>,
                $($pa: $pt),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}") $(,$p)*)),
                    $($pa),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _4level >](
                $($arg: $t),*,
                Path((one, two, three, four, $($p),*)): Path<(
                    String,
                    String,
                    String,
                    String,
                    $($crate::routes::macros::replace_expr!(($p) -> String)),*
                )>,
                $($pa: $pt),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}/{four}") $(,$p)*)),
                    $($pa),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _5level >](
                $($arg: $t),*,
                Path((one, two, three, four, five, $($p),*)): Path<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    $($crate::routes::macros::replace_expr!(($p) -> String)),*
                )>,
                $($pa: $pt),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}/{four}/{five}") $(,$p)*)),
                    $($pa),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _6level >](
                $($arg: $t),*,
                Path((one, two, three, four, five, six, $($p),*)): Path<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    $($crate::routes::macros::replace_expr!(($p) -> String)),*
                )>,
                $($pa: $pt),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}/{four}/{five}/{six}") $(,$p)*)),
                    $($pa),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _7level >](
                $($arg: $t),*,
                Path((one, two, three, four, five, six, seven, $($p),*)): Path<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    $($crate::routes::macros::replace_expr!(($p) -> String)),*
                )>,
                $($pa: $pt),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}/{four}/{five}/{six}/{seven}") $(,$p)*)),
                    $($pa),*
                )
                .await
            }
        }
    };
}

pub(crate) use route_7_levels;
pub(crate) use replace_expr;
pub(crate) use endpoint_fn_7_levels;
