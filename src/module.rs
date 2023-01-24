use pyo3::prelude::*;

use crate::fibonacci::*;
use crate::collatz::Collatz;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn poetry_rust_integration(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fib_recursive, m)?)?;

    m.add_class::<FibGenerator>()?;
    m.add_class::<Collatz>()?;

    Ok(())
}