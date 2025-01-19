use std::env;

/// Retrieves an environment variable or panics if it is not set.
pub fn get_env(key: &str) -> String {
    let error = format!("Please set the {key} environment variable");
    env::var(key).expect(&error)
}
