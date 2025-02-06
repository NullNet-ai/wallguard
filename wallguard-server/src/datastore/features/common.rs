use std::str::FromStr;
use tonic::{metadata::MetadataValue, Request};

use crate::datastore::DatastoreWrapper;
use nullnet_libdatastore::{Error as DSError, ErrorKind as DSErrorKind};

impl DatastoreWrapper {
    pub fn set_token_for_request<T>(request: &mut Request<T>, token: &str) -> Result<(), DSError> {
        let value = MetadataValue::from_str(token).map_err(|e| DSError {
            kind: DSErrorKind::ErrorRequestFailed,
            message: e.to_string(),
        })?;

        request.metadata_mut().insert("authorization", value);

        Ok(())
    }
}
