// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

/// Greets the user with a personalized message.
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A greeting string
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Application entry point - starts the Tauri event loop.
/// Note: This function cannot be unit tested as it starts a GUI.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_with_name() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! You've been greeted from Rust!");
    }

    #[test]
    fn test_greet_with_empty_name() {
        let result = greet("");
        assert_eq!(result, "Hello, ! You've been greeted from Rust!");
    }

    #[test]
    fn test_greet_with_special_characters() {
        let result = greet("Test<User>");
        assert_eq!(result, "Hello, Test<User>! You've been greeted from Rust!");
    }
}
