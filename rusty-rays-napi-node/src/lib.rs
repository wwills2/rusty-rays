mod build;

use napi_derive::napi;

#[napi]
mod bindings {
    use std::str::FromStr;
    use std::sync::Arc;

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

    #[napi(object)]
    pub struct Config {
        #[napi(ts_type = "\"trace\" | \"debug\" | \"info\" | \"warn\" | \"error\"")]
        pub log_level: String,
        pub log_files_dir: Option<String>,
        pub log_message_cache_overflow_limit: u32,
        pub max_render_threads: u32,
        pub loaded_from_file: bool,
    }

    /// Non-NAPI utility function
    fn core_config_to_js_config(config: rusty_rays_core::Config) -> Config {
        Config {
            log_level: config.log_level.to_string(),
            log_files_dir: config
                .log_files_dir
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            log_message_cache_overflow_limit: config.log_message_cache_overflow_limit as u32,
            max_render_threads: config.max_render_threads as u32,
            loaded_from_file: config.loaded_from_file,
        }
    }

    #[napi]
    pub fn get_default_config() -> Config {
        core_config_to_js_config(rusty_rays_core::Config::default())
    }

    #[napi]
    pub fn get_config() -> napi::Result<Config> {
        let config = rusty_rays_core::Config::get();
        Ok(core_config_to_js_config(config))
    }

    #[napi]
    pub async fn set_config(new_config: Config) -> napi::Result<()> {
        // run blocking set on a background thread as requested
        tokio::task::spawn_blocking(move || {
            let lower = new_config.log_level.as_str();
            if lower == "critical" {
                return Err("invalid log level: critical".to_string());
            }
            let level = rusty_rays_core::logger::Level::from_str(lower)
                .map_err(|_| format!("invalid log level: {}", lower))?;
            let log_files_dir = new_config
                .log_files_dir
                .map(|s| std::path::PathBuf::from(s));

            let config = rusty_rays_core::Config {
                log_level: level,
                log_files_dir,
                log_message_cache_overflow_limit: new_config.log_message_cache_overflow_limit
                    as usize,
                max_render_threads: new_config.max_render_threads as usize,
                loaded_from_file: new_config.loaded_from_file,
            };
            rusty_rays_core::Config::set(config)
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
        .map_err(|e| napi::Error::from_reason(e))?;
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
