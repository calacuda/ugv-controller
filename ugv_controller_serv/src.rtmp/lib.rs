use pyo3::prelude::*;

pub mod server;

/// A Python module implemented in Rust.
#[pymodule]
mod ugv_controller_serv {
    use log::*;
    use pyo3::prelude::*;
    use std::thread::{spawn, JoinHandle};

    use crate::server::server;

    // /// Formats the sum of two numbers as string.
    // #[pyfunction]
    // fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    //     Ok((a + b).to_string())
    // }

    #[pyclass]
    pub struct Server {
        _jh: JoinHandle<()>,
    }

    /// starts the server in a thread.
    #[pyfunction]
    fn start_server() -> PyResult<Server> {
        env_logger::init();

        Ok(Server {
            _jh: spawn(|| {
                if let Err(e) = server() {
                    error!("{e}")
                }
            }),
        })
    }
}
