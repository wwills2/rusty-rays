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
        ) -> napi::Result<Option<IntersectedObjectInfo>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner_tracer_clone = self.inner.clone();
            let result = tokio::task::spawn_blocking(move || {
                Ok::<Option<IntersectedObjectInfo>, String>(
                    inner_tracer_clone
                        .get_intersected_uuid_by_pixel_pos(x as usize, y as usize)
                        .map(|(u, t)| IntersectedObjectInfo {
                            uuid: u.to_string(),
                            object_type: t,
                        }),
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

            Ok(self
                .inner
                .get_all_spheres()
                .iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi(getter)]
        pub async fn get_all_cones(&self) -> napi::Result<HashMap<String, Cone>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(self
                .inner
                .get_all_cones()
                .iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi(getter)]
        pub async fn get_all_polygons(&self) -> napi::Result<HashMap<String, Polygon>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(self
                .inner
                .get_all_polygons()
                .iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi(getter)]
        pub async fn get_all_triangles(&self) -> napi::Result<HashMap<String, Triangle>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(self
                .inner
                .get_all_triangles()
                .iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect())
        }

        #[napi]
        pub async unsafe fn upsert_sphere(
            &mut self,
            sphere: Sphere,
        ) -> napi::Result<Option<Sphere>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner = match Arc::get_mut(&mut self.inner) {
                Some(inner) => inner,
                None => {
                    return Err(napi::Error::from_reason(
                        "Cannot modify model: multiple references exist",
                    ));
                }
            };

            let core_sphere = sphere.try_into()?;

            Ok(inner.upsert_sphere(core_sphere).map(|s| (&s).into()))
        }

        #[napi]
        pub async unsafe fn delete_sphere(&mut self, uuid: String) -> napi::Result<Option<Sphere>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner = match Arc::get_mut(&mut self.inner) {
                Some(inner) => inner,
                None => {
                    return Err(napi::Error::from_reason(
                        "Cannot modify model: multiple references exist",
                    ));
                }
            };

            let uuid =
                Uuid::from_str(&uuid).map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(inner.delete_sphere(uuid).map(|s| (&s).into()))
        }

        #[napi]
        pub async unsafe fn upsert_cone(&mut self, cone: Cone) -> napi::Result<Option<Cone>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner = match Arc::get_mut(&mut self.inner) {
                Some(inner) => inner,
                None => {
                    return Err(napi::Error::from_reason(
                        "Cannot modify model: multiple references exist",
                    ));
                }
            };

            let core_cone = cone.try_into()?;

            Ok(inner.upsert_cone(core_cone).map(|c| (&c).into()))
        }

        #[napi]
        pub async unsafe fn delete_cone(&mut self, uuid: String) -> napi::Result<Option<Cone>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner = match Arc::get_mut(&mut self.inner) {
                Some(inner) => inner,
                None => {
                    return Err(napi::Error::from_reason(
                        "Cannot modify model: multiple references exist",
                    ));
                }
            };

            let uuid =
                Uuid::from_str(&uuid).map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(inner.delete_cone(uuid).map(|c| (&c).into()))
        }

        #[napi]
        pub async unsafe fn upsert_polygon(
            &mut self,
            polygon: Polygon,
        ) -> napi::Result<Option<Polygon>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner = match Arc::get_mut(&mut self.inner) {
                Some(inner) => inner,
                None => {
                    return Err(napi::Error::from_reason(
                        "Cannot modify model: multiple references exist",
                    ));
                }
            };

            let core_polygon = polygon.try_into()?;

            Ok(inner.upsert_polygon(core_polygon).map(|p| (&p).into()))
        }

        #[napi]
        pub async unsafe fn delete_polygon(
            &mut self,
            uuid: String,
        ) -> napi::Result<Option<Polygon>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner = match Arc::get_mut(&mut self.inner) {
                Some(inner) => inner,
                None => {
                    return Err(napi::Error::from_reason(
                        "Cannot modify model: multiple references exist",
                    ));
                }
            };

            let uuid =
                Uuid::from_str(&uuid).map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(inner.delete_polygon(uuid).map(|p| (&p).into()))
        }

        #[napi]
        pub async unsafe fn upsert_triangle(
            &mut self,
            triangle: Triangle,
        ) -> napi::Result<Option<Triangle>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner = match Arc::get_mut(&mut self.inner) {
                Some(inner) => inner,
                None => {
                    return Err(napi::Error::from_reason(
                        "Cannot modify model: multiple references exist",
                    ));
                }
            };

            let core_triangle = triangle.try_into()?;

            Ok(inner.upsert_triangle(core_triangle).map(|t| (&t).into()))
        }

        #[napi]
        pub async unsafe fn delete_triangle(
            &mut self,
            uuid: String,
        ) -> napi::Result<Option<Triangle>> {
            let _acquired_permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let inner = match Arc::get_mut(&mut self.inner) {
                Some(inner) => inner,
                None => {
                    return Err(napi::Error::from_reason(
                        "Cannot modify model: multiple references exist",
                    ));
                }
            };

            let uuid =
                Uuid::from_str(&uuid).map_err(|e| napi::Error::from_reason(e.to_string()))?;

            Ok(inner.delete_triangle(uuid).map(|t| (&t).into()))
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
    pub struct Polygon {
        pub uuid: String,
        pub surface: String,
        pub plane_basis_vectors: (Coords, Coords),
        pub plane_sample_point: Coords,
        pub plane_normal: Coords,
        pub plane_projected_vertices: Vec<PlaneCoords2D>,
        pub point_in_polygon_inf_test_vector: PlaneCoords2D,
        pub projection_origin: Coords,
        pub vertices: Vec<Coords>,
    }

    impl From<&rusty_rays_core::Polygon> for Polygon {
        fn from(polygon: &rusty_rays_core::Polygon) -> Polygon {
            Polygon {
                uuid: polygon.uuid.to_string(),
                surface: polygon.surface.clone(),
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
                vertices: polygon.vertices.iter().map(|v| v.clone().into()).collect(),
            }
        }
    }

    impl TryFrom<Polygon> for rusty_rays_core::Polygon {
        type Error = napi::Error;

        fn try_from(polygon: Polygon) -> napi::Result<rusty_rays_core::Polygon> {
            Ok(rusty_rays_core::Polygon {
                uuid: Uuid::from_str(&polygon.uuid)
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?,
                surface: polygon.surface,
                plane: rusty_rays_core::Plane {
                    uuid: Uuid::new_v4(), // Generate a new UUID for the internal plane
                    basis_vectors: (
                        polygon.plane_basis_vectors.0.into(),
                        polygon.plane_basis_vectors.1.into(),
                    ),
                    sample_point: polygon.plane_sample_point.into(),
                    normal: polygon.plane_normal.into(),
                    surface: String::new(),
                },
                plane_projected_vertices: polygon
                    .plane_projected_vertices
                    .into_iter()
                    .map(|v| v.into())
                    .collect(),
                point_in_polygon_inf_test_vector: polygon.point_in_polygon_inf_test_vector.into(),
                projection_origin: polygon.projection_origin.into(),
                vertices: polygon.vertices.into_iter().map(|v| v.into()).collect(),
            })
        }
    }

    #[napi(object)]
    pub struct Triangle {
        pub uuid: String,
        pub surface: String,
        pub plane_basis_vectors: (Coords, Coords),
        pub plane_sample_point: Coords,
        pub plane_normal: Coords,
        pub vertex_1: Coords,
        pub vertex_2: Coords,
        pub vertex_3: Coords,
        pub edge_1: Coords,
        pub edge_2: Coords,
        pub edge_3: Coords,
        pub v1_normal: Coords,
        pub v2_normal: Coords,
        pub v3_normal: Coords,
        pub flat_shaded: bool,
        pub total_area: f64,
    }

    impl From<&rusty_rays_core::Triangle> for Triangle {
        fn from(triangle: &rusty_rays_core::Triangle) -> Triangle {
            Triangle {
                uuid: triangle.uuid.to_string(),
                surface: triangle.surface.clone(),
                plane_basis_vectors: (
                    triangle.plane.basis_vectors.0.clone().into(),
                    triangle.plane.basis_vectors.1.clone().into(),
                ),
                plane_sample_point: triangle.plane.sample_point.clone().into(),
                plane_normal: triangle.plane.normal.clone().into(),
                vertex_1: triangle.vertex_1.clone().into(),
                vertex_2: triangle.vertex_2.clone().into(),
                vertex_3: triangle.vertex_3.clone().into(),
                edge_1: triangle.edge_1.clone().into(),
                edge_2: triangle.edge_2.clone().into(),
                edge_3: triangle.edge_3.clone().into(),
                v1_normal: triangle.v1_normal.clone().into(),
                v2_normal: triangle.v2_normal.clone().into(),
                v3_normal: triangle.v3_normal.clone().into(),
                flat_shaded: triangle.flat_shaded,
                total_area: triangle.total_area,
            }
        }
    }

    impl TryFrom<Triangle> for rusty_rays_core::Triangle {
        type Error = napi::Error;

        fn try_from(triangle: Triangle) -> napi::Result<rusty_rays_core::Triangle> {
            Ok(rusty_rays_core::Triangle {
                uuid: Uuid::from_str(&triangle.uuid)
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?,
                surface: triangle.surface,
                plane: rusty_rays_core::Plane {
                    uuid: Uuid::new_v4(), // Generate a new UUID for the internal plane
                    basis_vectors: (
                        triangle.plane_basis_vectors.0.into(),
                        triangle.plane_basis_vectors.1.into(),
                    ),
                    sample_point: triangle.plane_sample_point.into(),
                    normal: triangle.plane_normal.into(),
                    surface: String::new(),
                },
                vertex_1: triangle.vertex_1.into(),
                vertex_2: triangle.vertex_2.into(),
                vertex_3: triangle.vertex_3.into(),
                edge_1: triangle.edge_1.into(),
                edge_2: triangle.edge_2.into(),
                edge_3: triangle.edge_3.into(),
                v1_normal: triangle.v1_normal.into(),
                v2_normal: triangle.v2_normal.into(),
                v3_normal: triangle.v3_normal.into(),
                flat_shaded: triangle.flat_shaded,
                total_area: triangle.total_area,
            })
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
