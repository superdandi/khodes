//! Entrada del host standalone de desarrollo.
//!
//! Solo se compila con la feature `standalone`. No es un formato distribuido.

use khodes::Plugin;

fn main() {
    truce_standalone::run::<Plugin>();
}
