use napi_derive::napi;

#[allow(unused)]
#[napi]
mod bindings {
    #![allow(dead_code)]
    use napi::bindgen_prelude::Buffer;
    use rusty_rays_core::Uuid;
    use std::collections::HashMap;
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
    pub struct IntersectedObjectInfo {
        pub uuid: String,
        #[napi(ts_type = "\"cone\" | \"sphere\" | \"polygon\" | \"triangle\"")]
        pub object_type: String,
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
    #[derive(Clone)]
    pub enum RenderEvent {
        Progress { percent: u8 },
        Finished { millis: u32 },
        WritingImage,
        Canceled { millis: u32 },
        Error { message: String },
    }

    impl From<rusty_rays_core::RenderEvent> for RenderEvent {
        fn from(core_event: rusty_rays_core::RenderEvent) -> Self {
            match core_event {
                rusty_rays_core::RenderEvent::Progress { percent } => {
                    RenderEvent::Progress { percent }
                }
                rusty_rays_core::RenderEvent::Finished { millis } => RenderEvent::Finished {
                    millis: millis.min(u32::MAX as u128) as u32,
                },
                rusty_rays_core::RenderEvent::Canceled { millis } => RenderEvent::Canceled {
                    millis: millis.min(u32::MAX as u128) as u32,
                },
                rusty_rays_core::RenderEvent::Error(s) => RenderEvent::Error { message: s },
            }
        }
    }

    #[napi]
    pub struct Tracer {
        inner: Arc<tokio::sync::Mutex<rusty_rays_core::Tracer>>,
        current_render_cancel: Arc<tokio::sync::Mutex<Option<rusty_rays_core::CancellationToken>>>,
    }

    #[napi]
    impl Tracer {
        #[napi(factory)]
        pub async fn create(model: &Model) -> napi::Result<Self> {
            let model_guard = model.inner.lock().await;
            let model_clone = (*model_guard).clone();
            let tracer = rusty_rays_core::Tracer::new(model_clone);

            Ok(Self {
                inner: Arc::new(tokio::sync::Mutex::new(tracer)),
                current_render_cancel: Arc::new(tokio::sync::Mutex::new(None)),
            })
        }

        #[napi]
        pub async fn render_to_image_buffer(
            &self,
            image_format: String,
            num_progress_blocks: i64,
            on_event: napi::threadsafe_function::ThreadsafeFunction<RenderEvent>,
        ) -> napi::Result<Buffer> {
            let inner = self.inner.clone();
            let on_event = Arc::new(on_event);
            let write_image = move |raw_render| {
                rusty_rays_core::write_render_to_image_buffer(image_format, &raw_render)
                    .map(Buffer::from)
                    .map_err(|e| e.to_string())
            };

            let cancel_token = rusty_rays_core::CancellationToken::default();
            self.current_render_cancel
                .lock()
                .await
                .replace(cancel_token.clone());

            let handle = Self::spawn_blocking_render_with_events(
                inner,
                cancel_token,
                num_progress_blocks,
                on_event,
                write_image,
            );

            let buf = handle
                .await
                .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
                .map_err(napi::Error::from_reason)?;

            self.clear_cancel_token().await;

            Ok(buf)
        }

        #[napi]
        pub async fn render_to_file(
            &self,
            output_file_path: String,
            num_progress_blocks: i64,
            on_event: napi::threadsafe_function::ThreadsafeFunction<RenderEvent>,
        ) -> napi::Result<()> {
            let inner = self.inner.clone();
            let on_event = Arc::new(on_event);
            let write_image = move |raw_render| {
                rusty_rays_core::write_render_to_file(&output_file_path.into(), &raw_render)
                    .map_err(|e| e.to_string())?;
                Ok(())
            };

            let cancel_token = rusty_rays_core::CancellationToken::default();
            self.current_render_cancel
                .lock()
                .await
                .replace(cancel_token.clone());

            let handle = Self::spawn_blocking_render_with_events(
                inner,
                cancel_token,
                num_progress_blocks,
                on_event,
                write_image,
            );

            handle
                .await
                .map_err(|e| napi::Error::from_reason(format!("task panicked: {e}")))?
                .map_err(napi::Error::from_reason)?;

            self.clear_cancel_token().await;

            Ok(())
        }

        #[napi]
        pub async fn cancel_render(&self) -> napi::Result<()> {
            if let Some(token) = self.current_render_cancel.lock().await.clone() {
                token.cancel();
            }
            self.clear_cancel_token().await;
            Ok(())
        }

        #[napi]
        pub async fn get_intersected_uuid_by_pixel_pos(
            &self,
            x: u32,
            y: u32,
        ) -> napi::Result<Option<IntersectedObjectInfo>> {
            let tracer_guard = self.inner.lock().await;
            let result = tracer_guard
                .get_intersected_uuid_by_pixel_pos(x as usize, y as usize)
                .map(|(u, t)| IntersectedObjectInfo {
                    uuid: u.to_string(),
                    object_type: t,
                });

            Ok(result)
        }

        fn spawn_blocking_render_with_events<T, F>(
            inner: Arc<tokio::sync::Mutex<rusty_rays_core::Tracer>>,
            cancel_token: rusty_rays_core::CancellationToken,
            num_progress_blocks: i64,
            on_event: Arc<napi::threadsafe_function::ThreadsafeFunction<RenderEvent>>,
            write_image: F,
        ) -> tokio::task::JoinHandle<Result<T, String>>
        where
            T: Send + 'static,
            F: FnOnce(Vec<Vec<rusty_rays_core::Color>>) -> Result<T, String> + Send + 'static,
        {
            tokio::task::spawn_blocking(move || -> Result<T, String> {
                // Channel that core uses to emit RenderEvent values.
                let (tx, rx) = std::sync::mpsc::channel::<rusty_rays_core::RenderEvent>();

                // Forward core events to JS on a dedicated listener thread.
                let on_event_listener = on_event.clone();
                let listener_handle = std::thread::spawn(move || {
                    let mut render_time_millis = 0;
                    for core_render_event in rx.iter() {
                        let render_event: RenderEvent = core_render_event.into();
                        if let RenderEvent::Finished { millis } = render_event {
                            // we're not done until the render has been written as an image
                            // do not send the finished event on render completion
                            render_time_millis = millis;
                        } else {
                            let _ = on_event_listener.call(
                                Ok(render_event.into()),
                                napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
                            );
                        }
                    }

                    render_time_millis
                });

                let tracer_guard = inner.blocking_lock();
                let raw_render = match tracer_guard.render(
                    Some(cancel_token),
                    Some(tx),
                    Some(num_progress_blocks as usize),
                ) {
                    Ok(render) => render,
                    Err(error) => {
                        // Ensure listener can exit cleanly.
                        let _ = listener_handle.join();
                        return Err(error.to_string());
                    }
                };

                // Join the listener (rx will be closed because tx was dropped when render returned).
                let listener_join_result = listener_handle.join();
                let render_time_millis = match listener_join_result {
                    Ok(t) => t,
                    Err(_) => return Err("Render listener thread panicked".to_string()),
                };

                let _ = on_event.call(
                    Ok(RenderEvent::WritingImage),
                    napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
                );

                // Do the endpoint-specific final step.
                let write_result = write_image(raw_render);

                let _ = on_event.call(
                    Ok(RenderEvent::Finished {
                        millis: render_time_millis,
                    }),
                    napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
                );

                write_result
            })
        }

        async fn clear_cancel_token(&self) {
            let _ = self.current_render_cancel.lock().await.take();
        }
    }

    #[napi]
    pub struct Model {
        inner: Arc<tokio::sync::Mutex<rusty_rays_core::Model>>,
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
                inner: Arc::new(tokio::sync::Mutex::new(model)),
            })
        }

        #[napi(factory)]
        pub fn from_string(input_string: String) -> napi::Result<Self> {
            let model = rusty_rays_core::Model::from_string(input_string)
                .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e.to_string()))?;
            Ok(Self {
                inner: Arc::new(tokio::sync::Mutex::new(model)),
            })
        }

        #[napi(getter)]
        pub async fn get_all_spheres(&self) -> napi::Result<HashMap<String, Sphere>> {
            let model_guard = self.inner.lock().await;

            Ok(model_guard
                .get_all_spheres()
                .iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi(getter)]
        pub async fn get_all_cones(&self) -> napi::Result<HashMap<String, Cone>> {
            let model_guard = self.inner.lock().await;

            Ok(model_guard
                .get_all_cones()
                .iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi(getter)]
        pub async fn get_all_polygons(&self) -> napi::Result<HashMap<String, Polygon>> {
            let model_guard = self.inner.lock().await;

            Ok(model_guard
                .get_all_polygons()
                .iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi(getter)]
        pub async fn get_all_triangles(&self) -> napi::Result<HashMap<String, Triangle>> {
            let model_guard = self.inner.lock().await;

            Ok(model_guard
                .get_all_triangles()
                .iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi]
        pub async fn upsert_sphere(&self, sphere: Sphere) -> napi::Result<Option<Sphere>> {
            let mut model_guard = self.inner.lock().await;
            Ok(model_guard
                .upsert_sphere(sphere.try_into()?)
                .map(|s| (&s).into()))
        }

        #[napi]
        pub async fn delete_sphere(&self, uuid: String) -> napi::Result<Option<Sphere>> {
            let uuid =
                Uuid::from_str(&uuid).map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let mut model_guard = self.inner.lock().await;
            Ok(model_guard.delete_sphere(uuid).map(|s| (&s).into()))
        }

        #[napi]
        pub async fn upsert_cone(&self, cone: Cone) -> napi::Result<Option<Cone>> {
            let mut model_guard = self.inner.lock().await;
            Ok(model_guard
                .upsert_cone(cone.try_into()?)
                .map(|c| (&c).into()))
        }

        #[napi]
        pub async fn delete_cone(&self, uuid: String) -> napi::Result<Option<Cone>> {
            let uuid =
                Uuid::from_str(&uuid).map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let mut model_guard = self.inner.lock().await;
            Ok(model_guard.delete_cone(uuid).map(|c| (&c).into()))
        }

        #[napi]
        pub async fn upsert_polygon(&self, polygon: Polygon) -> napi::Result<Option<Polygon>> {
            let mut model_guard = self.inner.lock().await;
            Ok(model_guard
                .upsert_polygon(polygon.try_into()?)
                .map(|p| (&p).into()))
        }

        #[napi]
        pub async fn delete_polygon(&self, uuid: String) -> napi::Result<Option<Polygon>> {
            let uuid =
                Uuid::from_str(&uuid).map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let mut model_guard = self.inner.lock().await;
            Ok(model_guard.delete_polygon(uuid).map(|p| (&p).into()))
        }

        #[napi]
        pub async fn upsert_triangle(&self, triangle: Triangle) -> napi::Result<Option<Triangle>> {
            let mut model_guard = self.inner.lock().await;
            Ok(model_guard
                .upsert_triangle(triangle.try_into()?)
                .map(|t| (&t).into()))
        }

        #[napi]
        pub async fn delete_triangle(&self, uuid: String) -> napi::Result<Option<Triangle>> {
            let mut model_guard = self.inner.lock().await;

            let uuid =
                Uuid::from_str(&uuid).map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(model_guard.delete_triangle(uuid).map(|t| (&t).into()))
        }

        #[napi(getter)]
        pub async fn get_all_surfaces(&self) -> napi::Result<Vec<Surface>> {
            let model_guard = self.inner.lock().await;

            Ok(model_guard
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

    impl TryFrom<Sphere> for rusty_rays_core::Sphere {
        type Error = napi::Error;

        fn try_from(sphere: Sphere) -> napi::Result<rusty_rays_core::Sphere> {
            Ok(rusty_rays_core::Sphere {
                uuid: Uuid::from_str(&sphere.uuid)
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?,
                surface: sphere.surface,
                radius: sphere.radius,
                position: sphere.position.into(),
            })
        }
    }

    #[napi(object)]
    pub struct Cone {
        pub uuid: String,
        pub surface: String,
        pub base_radius: f64,
        pub base: Coords,
        pub apex_radius: f64,
        pub apex: Coords,
    }

    impl From<&rusty_rays_core::Cone> for Cone {
        fn from(cone: &rusty_rays_core::Cone) -> Cone {
            Cone {
                uuid: cone.uuid.to_string(),
                surface: cone.surface.clone(),
                base_radius: cone.base_radius,
                base: cone.base.clone().into(),
                apex_radius: cone.apex_radius,
                apex: cone.apex.clone().into(),
            }
        }
    }

    impl TryFrom<Cone> for rusty_rays_core::Cone {
        type Error = napi::Error;

        fn try_from(cone: Cone) -> napi::Result<rusty_rays_core::Cone> {
            Ok(rusty_rays_core::Cone {
                uuid: Uuid::from_str(&cone.uuid)
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?,
                surface: cone.surface,
                base_radius: cone.base_radius,
                base: cone.base.into(),
                apex_radius: cone.apex_radius,
                apex: cone.apex.into(),
            })
        }
    }

    #[napi(object)]
    pub struct PlaneCoords2D {
        pub x: f64,
        pub y: f64,
    }

    impl From<PlaneCoords2D> for rusty_rays_core::PlaneCoords2D {
        fn from(coords: PlaneCoords2D) -> rusty_rays_core::PlaneCoords2D {
            rusty_rays_core::PlaneCoords2D {
                x: coords.x,
                y: coords.y,
            }
        }
    }

    #[napi(object)]
    pub struct PolygonDerived {
        pub plane_basis_vectors: (Coords, Coords),
        pub plane_sample_point: Coords,
        pub plane_normal: Coords,
        pub plane_projected_vertices: Vec<PlaneCoords2D>,
        pub point_in_polygon_inf_test_vector: PlaneCoords2D,
        pub projection_origin: Coords,
    }

    #[napi(object)]
    pub struct Polygon {
        pub uuid: String,
        pub surface: String,
        pub vertices: Vec<Coords>,
        pub derived: PolygonDerived,
    }

    impl From<&rusty_rays_core::Polygon> for Polygon {
        fn from(polygon: &rusty_rays_core::Polygon) -> Polygon {
            Polygon {
                uuid: polygon.uuid.to_string(),
                surface: polygon.surface.clone(),
                vertices: polygon.vertices.iter().map(|v| v.clone().into()).collect(),
                derived: PolygonDerived {
                    plane_basis_vectors: (
                        polygon.plane.basis_vectors.0.clone().into(),
                        polygon.plane.basis_vectors.1.clone().into(),
                    ),
                    plane_sample_point: polygon.plane.sample_point.clone().into(),
                    plane_normal: polygon.plane.normal.clone().into(),
                    plane_projected_vertices: polygon
                        .plane_projected_vertices
                        .iter()
                        .map(|v| PlaneCoords2D { x: v.x, y: v.y })
                        .collect(),
                    point_in_polygon_inf_test_vector: PlaneCoords2D {
                        x: polygon.point_in_polygon_inf_test_vector.x,
                        y: polygon.point_in_polygon_inf_test_vector.y,
                    },
                    projection_origin: polygon.projection_origin.clone().into(),
                },
            }
        }
    }

    impl TryFrom<Polygon> for rusty_rays_core::Polygon {
        type Error = napi::Error;

        fn try_from(polygon: Polygon) -> napi::Result<rusty_rays_core::Polygon> {
            let mut core_polygon = rusty_rays_core::Polygon::new(
                polygon.vertices.into_iter().map(|v| v.into()).collect(),
                polygon.surface,
            )
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            core_polygon.uuid = Uuid::from_str(&polygon.uuid)
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(core_polygon)
        }
    }

    #[napi(object)]
    pub struct TriangleDerived {
        pub plane_basis_vectors: (Coords, Coords),
        pub plane_sample_point: Coords,
        pub plane_normal: Coords,
        pub edge_1: Coords,
        pub edge_2: Coords,
        pub edge_3: Coords,
        pub flat_shaded: bool,
        pub total_area: f64,
    }

    #[napi(object)]
    pub struct Triangle {
        pub uuid: String,
        pub surface: String,
        pub vertex_1: Coords,
        pub vertex_2: Coords,
        pub vertex_3: Coords,
        pub v1_normal: Option<Coords>,
        pub v2_normal: Option<Coords>,
        pub v3_normal: Option<Coords>,
        pub derived: TriangleDerived,
    }

    impl From<&rusty_rays_core::Triangle> for Triangle {
        fn from(triangle: &rusty_rays_core::Triangle) -> Triangle {
            Triangle {
                uuid: triangle.uuid.to_string(),
                surface: triangle.surface.clone(),
                vertex_1: triangle.vertex_1.clone().into(),
                vertex_2: triangle.vertex_2.clone().into(),
                vertex_3: triangle.vertex_3.clone().into(),
                v1_normal: Some(triangle.v1_normal.clone().into()),
                v2_normal: Some(triangle.v2_normal.clone().into()),
                v3_normal: Some(triangle.v3_normal.clone().into()),
                derived: TriangleDerived {
                    plane_basis_vectors: (
                        triangle.plane.basis_vectors.0.clone().into(),
                        triangle.plane.basis_vectors.1.clone().into(),
                    ),
                    plane_sample_point: triangle.plane.sample_point.clone().into(),
                    plane_normal: triangle.plane.normal.clone().into(),
                    edge_1: triangle.edge_1.clone().into(),
                    edge_2: triangle.edge_2.clone().into(),
                    edge_3: triangle.edge_3.clone().into(),
                    flat_shaded: triangle.flat_shaded,
                    total_area: triangle.total_area,
                },
            }
        }
    }

    impl TryFrom<Triangle> for rusty_rays_core::Triangle {
        type Error = napi::Error;

        fn try_from(triangle: Triangle) -> napi::Result<rusty_rays_core::Triangle> {
            let mut core_triangle = rusty_rays_core::Triangle::new(
                triangle.vertex_1.into(),
                triangle.vertex_2.into(),
                triangle.vertex_3.into(),
                triangle.v1_normal.map(|n| n.into()),
                triangle.v2_normal.map(|n| n.into()),
                triangle.v3_normal.map(|n| n.into()),
                triangle.surface,
            )
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
            core_triangle.uuid = Uuid::from_str(triangle.uuid.as_str())
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            Ok(core_triangle)
        }
    }

    impl From<Color> for rusty_rays_core::Color {
        fn from(color: Color) -> rusty_rays_core::Color {
            rusty_rays_core::Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
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

    impl From<Surface> for rusty_rays_core::Surface {
        fn from(surface: Surface) -> rusty_rays_core::Surface {
            rusty_rays_core::Surface {
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

    impl From<Coords> for rusty_rays_core::Coords {
        fn from(coords: Coords) -> rusty_rays_core::Coords {
            rusty_rays_core::Coords {
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
        pub a: f64,
    }

    impl From<rusty_rays_core::Color> for Color {
        fn from(color: rusty_rays_core::Color) -> Color {
            Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            }
        }
    }
}
