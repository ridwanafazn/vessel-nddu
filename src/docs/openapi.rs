//! Root OpenAPI document.
//!
//! UI tersedia di `/swagger-ui/`, spesifikasi JSON di `/api-docs/openapi.json`.

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Vessel NDDU API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Ship sensor simulator & NMEA 0183 / MQTT / WebSocket gateway.\n\n\
            **Kemampuan Utama:**\n\
            - **REST API**: Kontrol & konfigurasi simulator sensor kapal (GPS, Gyro, Weather, Compass, Speed, Depth)\n\
            - **WebSocket**: Real-time updates via `wss://vessel-nddu.onrender.com/ws` (tiap 1 detik)\n\
            - **MQTT**: Per-sensor JSON & NMEA per sentence di `vessel/<sensor>/...`\n\
            - **TCP**: NMEA streaming outbound client (opsional, disabled by default)\n\n\
            **Format Response GPS:**\n\
            - Default `GET /api/gps` → NMEA 0183 text (`$GPRMC,...`)\n\
            - Tambah `?format=json` untuk JSON (`{message, data: {...}}`)\n\n\
            **Autentikasi:**\n\
            - Login: `POST /api/auth/login` dengan `{\"username\":\"admin\",\"password\":\"admin\"}`\n\
            - Saat ini menggunakan hardcoded credentials (demo only)\n\n\
            **Deploy:**\n\
            - Gratis di Render Free (Docker, 512 MB RAM, WebSocket support)\n\
            - Auto-deploy on push ke `main` branch\n\n\
            Default `GET /api/gps` returns NMEA text; add `?format=json` for JSON."
    ),
    tags(
        (name = "Auth", description = "Autentikasi — gunakan `admin` / `admin`"),
        (name = "GPS", description = "Simulator GPS & NMEA (GPRMC/GPGLL/GPVTG/GPZDA)"),
        (name = "Gyro", description = "Simulator gyro/attitude (HDT/HDG/ROT/XDR)"),
        (name = "Weather", description = "Simulator cuaca gabungan"),
        (name = "Temperature", description = "Sensor suhu individual"),
        (name = "Humidity", description = "Sensor kelembapan individual"),
        (name = "Wind", description = "Sensor angin individual"),
        (name = "Pressure", description = "Sensor tekanan individual"),
        (name = "Compass", description = "Kompas magnetik"),
        (name = "Speed", description = "Speed log (VHW)"),
        (name = "Depth", description = "Depth sounder (DBT/DPT/DBK)"),
        (name = "Navigation", description = "Config bersama speed & depth"),
        (name = "MQTT", description = "Konfigurasi MQTT global"),
        (name = "Health", description = "Liveness & readiness check"),
        (name = "WebSocket", description = "Real-time updates via WebSocket"),
    ),
    paths(
        crate::controllers::auth_controller::login,

        crate::controllers::gps_controller::get_config,
        crate::controllers::gps_controller::post_config,
        crate::controllers::gps_controller::delete_config,
        crate::controllers::gps_controller::create_gps,
        crate::controllers::gps_controller::get_gps,
        crate::controllers::gps_controller::update_gps,
        crate::controllers::gps_controller::delete_gps,

        crate::controllers::gyro_controller::get_config,
        crate::controllers::gyro_controller::post_config,
        crate::controllers::gyro_controller::delete_config,
        crate::controllers::gyro_controller::create_gyro,
        crate::controllers::gyro_controller::get_gyro,
        crate::controllers::gyro_controller::update_gyro,
        crate::controllers::gyro_controller::delete_gyro,

        crate::controllers::weather_controller::get_config,
        crate::controllers::weather_controller::post_config,
        crate::controllers::weather_controller::delete_config,
        crate::controllers::weather_controller::create_weather,
        crate::controllers::weather_controller::get_weather,
        crate::controllers::weather_controller::update_weather,
        crate::controllers::weather_controller::delete_weather,

        crate::controllers::temperature_controller::create_temperature,
        crate::controllers::temperature_controller::get_temperature,
        crate::controllers::temperature_controller::update_temperature,
        crate::controllers::temperature_controller::delete_temperature,

        crate::controllers::humidity_controller::create_humidity,
        crate::controllers::humidity_controller::get_humidity,
        crate::controllers::humidity_controller::update_humidity,
        crate::controllers::humidity_controller::delete_humidity,

        crate::controllers::wind_controller::create_wind,
        crate::controllers::wind_controller::get_wind,
        crate::controllers::wind_controller::update_wind,
        crate::controllers::wind_controller::delete_wind,

        crate::controllers::pressure_controller::create_pressure,
        crate::controllers::pressure_controller::get_pressure,
        crate::controllers::pressure_controller::update_pressure,
        crate::controllers::pressure_controller::delete_pressure,

        crate::controllers::navigation_controller::get_config,
        crate::controllers::navigation_controller::post_config,
        crate::controllers::navigation_controller::delete_config,

        crate::controllers::speed_controller::create_speed,
        crate::controllers::speed_controller::get_speed,
        crate::controllers::speed_controller::update_speed,
        crate::controllers::speed_controller::delete_speed,

        crate::controllers::depth_controller::create_depth,
        crate::controllers::depth_controller::get_depth,
        crate::controllers::depth_controller::update_depth,
        crate::controllers::depth_controller::delete_depth,

        crate::controllers::compass_controller::get_config,
        crate::controllers::compass_controller::post_config,
        crate::controllers::compass_controller::delete_config,
        crate::controllers::compass_controller::get_compass,
        crate::controllers::compass_controller::post_compass,
        crate::controllers::compass_controller::patch_compass,
        crate::controllers::compass_controller::delete_compass,

        crate::controllers::mqtt_controller::get_config,
        crate::controllers::mqtt_controller::patch_config,
        crate::controllers::health_controller::health,
    ),
    components(schemas(
        crate::data::auth_data::LoginRequest,
        crate::data::auth_data::LoginResponse,

        crate::data::gps_data::GpsState,
        crate::data::gps_data::GpsConfig,
        crate::data::gps_data::CreateGpsRequest,
        crate::data::gps_data::UpdateGpsRequest,
        crate::data::gps_data::UpdateGpsConfigRequest,

        crate::data::gyro_data::GyroState,
        crate::data::gyro_data::GyroConfig,
        crate::data::gyro_data::CreateGyroRequest,
        crate::data::gyro_data::UpdateGyroRequest,
        crate::data::gyro_data::UpdateGyroConfigRequest,

        crate::data::weather_data::WeatherState,
        crate::data::weather_data::WeatherConfig,
        crate::data::weather_data::CreateWeatherRequest,
        crate::data::weather_data::UpdateWeatherRequest,
        crate::data::weather_data::UpdateWeatherConfigRequest,
        crate::data::weather_data::TemperatureState,
        crate::data::weather_data::HumidityState,
        crate::data::weather_data::WindState,
        crate::data::weather_data::PressureState,

        crate::data::navigation_data::SpeedState,
        crate::data::navigation_data::DepthState,
        crate::data::navigation_data::NavigationConfig,
        crate::data::navigation_data::CreateSpeedRequest,
        crate::data::navigation_data::UpdateSpeedRequest,
        crate::data::navigation_data::CreateDepthRequest,
        crate::data::navigation_data::UpdateDepthRequest,
        crate::data::navigation_data::UpdateNavigationConfigRequest,

        crate::data::compass_data::CompassConfig,
        crate::data::compass_data::CompassState,

        crate::controllers::temperature_controller::CreateTemperatureRequest,
        crate::controllers::temperature_controller::UpdateTemperatureRequest,
        crate::controllers::humidity_controller::CreateHumidityRequest,
        crate::controllers::humidity_controller::UpdateHumidityRequest,
        crate::controllers::wind_controller::CreateWindRequest,
        crate::controllers::wind_controller::UpdateWindRequest,
        crate::controllers::pressure_controller::CreatePressureRequest,
        crate::controllers::pressure_controller::UpdatePressureRequest,
        crate::controllers::compass_controller::CreateCompassRequest,
        crate::controllers::compass_controller::UpdateCompassRequest,
        crate::controllers::compass_controller::UpdateCompassConfigRequest,
        crate::controllers::mqtt_controller::UpdateMqttConfigRequest,

        crate::docs::responses::MessageResponse,
        crate::docs::responses::GpsResponse,
        crate::docs::responses::GpsConfigResponse,
        crate::docs::responses::GyroResponse,
        crate::docs::responses::GyroConfigResponse,
        crate::docs::responses::WeatherResponse,
        crate::docs::responses::WeatherConfigResponse,
        crate::docs::responses::TemperatureData,
        crate::docs::responses::TemperatureResponse,
        crate::docs::responses::HumidityData,
        crate::docs::responses::HumidityResponse,
        crate::docs::responses::WindData,
        crate::docs::responses::WindResponse,
        crate::docs::responses::PressureData,
        crate::docs::responses::PressureResponse,
        crate::docs::responses::SpeedResponse,
        crate::docs::responses::DepthResponse,
        crate::docs::responses::NavigationConfigResponse,
        crate::docs::responses::CompassData,
        crate::docs::responses::CompassResponse,
        crate::docs::responses::CompassConfigResponse,
        crate::docs::responses::MqttConfigData,
        crate::docs::responses::MqttConfigResponse,
        crate::controllers::health_controller::HealthData,
        crate::controllers::health_controller::HealthResponse,
    ))
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_contains_all_api_groups() {
        let doc = ApiDoc::openapi();
        let paths = &doc.paths.paths;

        for expected in [
            "/api/auth/login",
            "/api/gps",
            "/api/gps/config",
            "/api/gyro",
            "/api/gyro/config",
            "/api/weather",
            "/api/temperature",
            "/api/humidity",
            "/api/wind",
            "/api/pressure",
            "/api/compass",
            "/api/speed",
            "/api/depth",
            "/api/navigation/config",
            "/api/mqtt/config",
            "/health",
        ] {
            assert!(paths.contains_key(expected), "missing path {expected}");
        }

        assert!(doc.components.is_some(), "components/schemas missing");
    }
}
