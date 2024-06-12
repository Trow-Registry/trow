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

/// Macro to quickly write functions for image names several layers depp
/// eg: `nvidia/dcgm/whatever`
macro_rules! endpoint_fn_7_levels {
    ($fn_name:ident($($arg:ident: $arg_ty:ty),+; path: [image_name $(,$path_param:ident: $path_param_ty:ty)*] $(,$post_arg:ident: $post_arg_ty:ty)*) -> $ret:ty) => {
        paste::item! {
            pub async fn [< $fn_name _2level >](
                $($arg: $arg_ty),*,
                Path((one, two, $($path_param),*)): Path<(
                    String,
                    String,
                    $($path_param_ty),*
                )>,
                $($post_arg: $post_arg_ty),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}") $(,$path_param)*)),
                    $($post_arg),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _3level >](
                $($arg: $arg_ty),*,
                Path((one, two, three, $($path_param),*)): Path<(
                    String,
                    String,
                    String,
                    $($path_param_ty),*
                )>,
                $($post_arg: $post_arg_ty),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}") $(,$path_param)*)),
                    $($post_arg),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _4level >](
                $($arg: $arg_ty),*,
                Path((one, two, three, four, $($path_param),*)): Path<(
                    String,
                    String,
                    String,
                    String,
                    $($path_param_ty),*
                )>,
                $($post_arg: $post_arg_ty),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}/{four}") $(,$path_param)*)),
                    $($post_arg),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _5level >](
                $($arg: $arg_ty),*,
                Path((one, two, three, four, five, $($path_param),*)): Path<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    $($path_param_ty),*
                )>,
                $($post_arg: $post_arg_ty),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}/{four}/{five}") $(,$path_param)*)),
                    $($post_arg),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _6level >](
                $($arg: $arg_ty),*,
                Path((one, two, three, four, five, six, $($path_param),*)): Path<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    $($path_param_ty),*
                )>,
                $($post_arg: $post_arg_ty),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}/{four}/{five}/{six}") $(,$path_param)*)),
                    $($post_arg),*
                )
                .await
            }
        }
        paste::item! {
            pub async fn [< $fn_name _7level >](
                $($arg: $arg_ty),*,
                Path((one, two, three, four, five, six, seven, $($path_param),*)): Path<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    $($path_param_ty),*
                )>,
                $($post_arg: $post_arg_ty),*
            ) -> $ret {
                $fn_name(
                    $($arg),*,
                    Path((format!("{one}/{two}/{three}/{four}/{five}/{six}/{seven}") $(,$path_param)*)),
                    $($post_arg),*
                )
                .await
            }
        }
    };
}

pub(crate) use {endpoint_fn_7_levels, route_7_levels};
