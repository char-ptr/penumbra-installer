#[cfg(windows)]
extern crate windres;
#[cfg(windows)]
use windres::Build;
fn main() {
    #[cfg(windows)]
    Build::new().compile("pi.rc").unwrap()
}
