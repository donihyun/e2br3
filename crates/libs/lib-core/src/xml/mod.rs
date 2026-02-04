pub mod error;
pub mod exporter;
pub mod importer;
pub mod parser;
pub mod types;
pub mod validator;

pub use error::Error;
pub type Result<T> = core::result::Result<T, Error>;

pub use importer::{import_e2b_xml, XmlImportRequest};
pub use parser::parse_e2b_xml;
pub use types::ParsedE2b;
pub use types::{XmlImportResult, XmlValidationError, XmlValidationReport};
pub use validator::{validate_e2b_xml, XmlValidatorConfig};
pub use exporter::export_case_xml;
