pub mod bus;
pub mod event;
pub mod serial;
pub mod transport;
pub mod discovery;

pub fn get_all_event_message_topics() -> Vec<String> {
    vec![
        "com_input".to_string(),
        "sensor_data".to_string(),
        "navigation".to_string(),
        "pressure_sensor".to_string(),
        "network".to_string(),
        "system_status".to_string(),
        "llm_response".to_string(),
        "control".to_string(),
        "monitoring".to_string(),
        "device_discovered".to_string(),
        "device_registration".to_string(),
        "device_registered".to_string(),

        // static for now... code smell! TODO FIX AND INFER
        "arduino_sensor_001".to_string(),
        "a01".to_string(),
    ]
}
