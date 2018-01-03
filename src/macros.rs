/// A simple macro used to stub out a function that is not implemented
macro_rules! not_implemented {
    () => {{
        warn!("Function is not implemented");
        use util;
        Err(util::std_err("Not implemented"))
    }}
}
