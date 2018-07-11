extern crate rand;

#[cfg(not(target_arch = "wasm32"))]
macro_rules! using {
    ($t:ident) => (pub mod $t;)
}

#[cfg(target_arch = "wasm32")]
macro_rules! using {
    ($t:ident) => (mod $t;)
}

mod common;
using!(kakuro);
using!(format);
using!(grid_loop);
using!(slitherlink);
using!(numberlink);
using!(endview);

#[cfg(not(target_arch = "wasm32"))]
pub use common::*;

#[cfg(target_arch = "wasm32")]
use common::*;

#[cfg(target_arch = "wasm32")]
mod js;

#[cfg(target_arch = "wasm32")]
pub use js::*;
