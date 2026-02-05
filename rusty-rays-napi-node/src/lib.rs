mod build;

use napi_derive::napi;

#[napi]
mod bindings {
    use napi::bindgen_prelude::Buffer;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Arc;
    use uuid::Uuid;

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
            let log_files_dir = new_config.log_files_dir.map(std::path::PathBuf::from);

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
        .map_err(napi::Error::from_reason)?;
        Ok(())
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
        pub async fn render_to_image_buffer(&self, image_format: String) -> napi::Result<Buffer> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner_tracer_clone = self.inner.clone();
            let raw_image = tokio::task::spawn_blocking(move || {
                let raw_render = match inner_tracer_clone.render() {
                    Ok(render) => render,
                    Err(error) => return Err(error.to_string()),
                };

                match rusty_rays_core::write_render_to_image_buffer(image_format, &raw_render) {
                    Ok(serialized_render) => Ok(Buffer::from(serialized_render)),
                    Err(error) => Err(error.to_string()),
                }
            })
            .await
            .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
            .map_err(napi::Error::from_reason)?;

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
                    Err(error) => Err(error.to_string()),
                }
            })
            .await
            .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
            .map_err(napi::Error::from_reason)?;

            Ok(())
        }

        #[napi]
        pub async fn get_intersected_uuid_by_pixel_pos(
            &self,
            x: u32,
            y: u32,
        ) -> napi::Result<Option<String>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner_tracer_clone = self.inner.clone();
            let result = tokio::task::spawn_blocking(move || {
                Ok::<Option<String>, String>(
                    inner_tracer_clone
                        .get_intersected_uuid_by_pixel_pos(x as usize, y as usize)
                        .map(|u| u.to_string()),
                )
            })
            .await
            .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
            .map_err(napi::Error::from_reason)?;

            Ok(result)
        }
    }

    #[napi]
    pub struct Model {
        inner: Arc<rusty_rays_core::Model>,
        semaphore: Arc<tokio::sync::Semaphore>,
    }

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

        #[napi(getter)]
        pub async fn get_all_spheres(&self) -> napi::Result<HashMap<String, Sphere>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let foo = self.inner.get_all_spheres();

            Ok(self
                .inner
                .get_all_spheres()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi(getter)]
        pub async fn get_all_surfaces(&self) -> napi::Result<Vec<Surface>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(self
                .inner
                .surfaces
                .values()
                .map(|surface| surface.clone().into())
                .collect())
        }
    }

    #[napi(object)]
    pub struct Sphere {
        pub uuid: String,
        pub surface: String,
        pub radius: f64,
        pub position: Coords,
    }

    impl From<&rusty_rays_core::Sphere> for Sphere {
        fn from(sphere: &rusty_rays_core::Sphere) -> Sphere {
            Sphere {
                uuid: sphere.uuid.to_string(),
                surface: sphere.surface.clone(),
                radius: sphere.radius,
                position: sphere.position.clone().into(),
            }
        }
    }

    #[napi(object)]
    pub struct Surface {
        pub name: String,
        pub ambient: Color,
        pub diffuse: Color,
        pub specular: Color,
        pub specpow: f64,
        pub reflect: f64,
    }

    impl From<rusty_rays_core::Surface> for Surface {
        fn from(surface: rusty_rays_core::Surface) -> Surface {
            Surface {
                name: surface.name,
                ambient: surface.ambient.into(),
                diffuse: surface.diffuse.into(),
                specular: surface.specular.into(),
                specpow: surface.specpow,
                reflect: surface.reflect,
            }
        }
    }

    #[napi(object)]
    pub struct Coords {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }

    impl From<rusty_rays_core::Coords> for Coords {
        fn from(coords: rusty_rays_core::Coords) -> Coords {
            Coords {
                x: coords.x,
                y: coords.y,
                z: coords.z,
            }
        }
    }

    #[napi(object)]
    pub struct Color {
        pub r: f64,
        pub g: f64,
        pub b: f64,
    }

    impl From<rusty_rays_core::Color> for Color {
        fn from(color: rusty_rays_core::Color) -> Color {
            Color {
                r: color.r,
                g: color.g,
                b: color.b,
            }
        }
    }
}
