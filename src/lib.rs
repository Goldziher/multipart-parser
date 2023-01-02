mod content_header;
mod multipart;

use pyo3::prelude::*;
use std::collections::HashMap;

pub use content_header::parse_content_header as _parse_content_header;
pub use multipart::{parse_multipart_form_data as _parse_multipart_form_data, Field};

#[pyfunction]
#[pyo3(text_signature = "(header, /)")]
fn parse_content_header(header: &str) -> PyResult<(String, HashMap<String, String>)> {
    Ok(_parse_content_header(header))
}

#[pyfunction]
#[pyo3(text_signature = "(body, boundary, charset, /)")]
fn parse_multipart_form_data(
    body: &[u8],
    boundary: &[u8],
    charset: &[u8],
) -> PyResult<HashMap<String, Field>> {
    Ok(_parse_multipart_form_data(body, boundary, charset))
}

#[pymodule]
fn fast_multipart_parser(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_content_header, m)?)?;
    m.add_function(wrap_pyfunction!(parse_multipart_form_data, m)?)?;

    Ok(())
}
