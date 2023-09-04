#![deny(
    non_ascii_idents,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_must_use,
    clippy::unwrap_used
)]
pub mod cfg;
pub mod db;
pub mod handlers;
pub mod model;
pub mod schema;
pub mod services;
pub mod token;
pub mod usd_price;
