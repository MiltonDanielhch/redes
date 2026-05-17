//! Ubicación: `crates/infrastructure/src/openapi.rs`
//!
//! Descripción: OpenAPI specification con Utoipa.
//!
//! ADRs relacionados: 0003 (Axum), 0016 (OpenAPI)

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::sede_handler::list_sedes,
        crate::handlers::sede_handler::get_sede,
        crate::handlers::sede_handler::create_sede,
        crate::handlers::sede_handler::update_sede,
        crate::handlers::device_handler::list_devices,
        crate::handlers::device_handler::get_device,
        crate::handlers::device_handler::create_device,
        crate::handlers::device_handler::update_device,
        crate::handlers::device_handler::delete_device,
    ),
    components(
        schemas(
            crate::dto::sede::SedeResponse,
            crate::dto::sede::CreateSedeRequest,
            crate::dto::sede::UpdateSedeRequest,
            crate::dto::device::DeviceResponse,
            crate::dto::device::CreateDeviceRequest,
            crate::dto::device::UpdateDeviceRequest,
            crate::dto::error::ApiErrorResponse,
        )
    ),
    tags(
        (name = "sedes", description = "Gestión de sedes regionales del Beni"),
        (name = "devices", description = "Gestión de dispositivos de red"),
        (name = "health", description = "Health checks de la API")
    ),
    info(
        title = "API Redes - Monitoreo de Infraestructura Regional",
        version = "0.1.0",
        description = "API para el sistema de monitoreo de infraestructura de red de la Gobernación del Beni. Permite gestionar sedes regionales, dispositivos de red, métricas y alertas.",
        contact(
            name = "API Support",
            email = "soporte@redes.gob.bo"
        )
    )
)]
pub struct ApiDoc;