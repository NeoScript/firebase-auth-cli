use anyhow::Result;
use error_stack::Report;
use rs_firebase_admin_sdk::client::error::ApiClientError;

pub trait IntoAnyhow<T> {
    fn into_anyhow(self) -> Result<T>;
}

impl<T> IntoAnyhow<T> for std::result::Result<T, Report<ApiClientError>> {
    fn into_anyhow(self) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("{e}"))
    }
}
