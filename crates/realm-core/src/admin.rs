use std::collections::HashSet;
use std::sync::OnceLock;

pub fn is_admin(username: &str) -> bool {
    static ADMIN_USERS: OnceLock<HashSet<String>> = OnceLock::new();

    let admins = ADMIN_USERS.get_or_init(|| {
        std::env::var("ADMIN_USERS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase)
            .collect()
    });

    admins.contains(&username.to_lowercase())
}