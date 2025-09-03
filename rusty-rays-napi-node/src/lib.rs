mod build;

use napi_derive::napi;
use std::sync::Arc;

#[napi]
mod bindings {
    use super::*;
    #[napi]
    pub struct Model {
        inner: Arc<rusty_rays_core::Model>,
        semaphore: Arc<tokio::sync::Semaphore>,
    }

    //todo add setters and getters for fields
    #[napi]
    impl Model {
        #[napi(factory)]
        pub async fn from_file_path(path: String) -> napi::Result<Self> {
            // put fs read on task thread
            let model = tokio::task::spawn_blocking(move || {
                rusty_rays_core::Model::from_file_path(path.into())
            })
            .await
            .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(Self {
                inner: Arc::new(model),
                semaphore: Arc::new(tokio::sync::Semaphore::new(1)),
            })
        }

        #[napi(factory)]
        pub fn from_string(input_string: String) -> napi::Result<Self> {
            let model = rusty_rays_core::Model::from_string(input_string)
                .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e.to_string()))?;
            Ok(Self {
                inner: Arc::new(model),
                semaphore: Arc::new(tokio::sync::Semaphore::new(1)),
            })
        }
    }
}
