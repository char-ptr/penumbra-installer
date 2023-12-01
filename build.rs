extern crate windres;
use windres::Build;
fn main() {
    Build::new().compile("pi.rc").unwrap()
}

