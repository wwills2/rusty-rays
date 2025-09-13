mod build;

use napi_derive::napi;
use std::sync::Arc;

#[napi]
mod bindings {
    use super::*;

    #[napi]
    pub fn log_error(message: String) -> napi::Result<()> {
        rusty_rays_core::logger::error!(rusty_rays_core::logger::LOG, "{}", message);
        Ok(())
    }

    #[napi]
    pub fn log_warn(message: String) -> napi::Result<()> {
        rusty_rays_core::logger::warn!(rusty_rays_core::logger::LOG, "{}", message);
        Ok(())
    }

    #[napi]
    pub fn log_info(message: String) -> napi::Result<()> {
        rusty_rays_core::logger::info!(rusty_rays_core::logger::LOG, "{}", message);
        Ok(())
    }

    #[napi]
    pub fn log_debug(message: String) -> napi::Result<()> {
        rusty_rays_core::logger::debug!(rusty_rays_core::logger::LOG, "{}", message);
        Ok(())
    }

    #[napi]
    pub fn log_trace(message: String) -> napi::Result<()> {
        rusty_rays_core::logger::trace!(rusty_rays_core::logger::LOG, "{}", message);
        Ok(())
    }

    #[napi]
    pub fn shutdown_logger() -> napi::Result<()> {
        rusty_rays_core::logger::shutdown_logger();
        Ok(())
    }

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

    #[napi]
    pub struct Tracer {
        inner: Arc<rusty_rays_core::Tracer>,
        semaphore: Arc<tokio::sync::Semaphore>,
    }

    #[napi]
    impl Tracer {
        #[napi(constructor)]
        pub fn new(model: &Model) -> napi::Result<Self> {
            let model_clone = (*model.inner).clone();
            let tracer = rusty_rays_core::Tracer::new(model_clone);

            Ok(Self {
                inner: Arc::new(tracer),
                semaphore: Arc::new(tokio::sync::Semaphore::new(1)),
            })
        }

        #[napi]
        pub async fn render_to_serialized(&self) -> napi::Result<Vec<u8>> {
            let _acquired_permit = self.semaphore.clone().acquire_owned().await;

            let inner_tracer_clone = self.inner.clone();
            let raw_image = tokio::task::spawn_blocking(move || {
                let raw_render = match inner_tracer_clone.render() {
                    Ok(render) => render,
                    Err(error) => return Err(error.to_string()),
                };

                match rusty_rays_core::serialize_raw_render_to_blob(&raw_render) {
                    Ok(serialized_render) => Ok(serialized_render),
                    Err(error) => return Err(error.to_string()),
                }
            })
            .await
            .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
            .map_err(|e| napi::Error::from_reason(e))?;

            Ok(raw_image)
        }

        #[napi]
        pub async fn render_to_file(&self, output_file_path: String) -> napi::Result<()> {
            let _acquired_permit = self.semaphore.clone().acquire_owned().await;

            let inner_tracer_clone = self.inner.clone();
            tokio::task::spawn_blocking(move || {
                let raw_render = match inner_tracer_clone.render() {
                    Ok(render) => render,
                    Err(error) => return Err(error.to_string()),
                };

                match rusty_rays_core::write_render_to_file(&output_file_path.into(), &raw_render) {
                    Ok(serialized_render) => Ok(serialized_render),
                    Err(error) => return Err(error.to_string()),
                }
            })
            .await
            .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
            .map_err(|e| napi::Error::from_reason(e))?;

            Ok(())
        }
    }
}
