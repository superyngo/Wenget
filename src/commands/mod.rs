//! Command implementations for WenPM

pub mod add;
pub mod delete;
pub mod info;
pub mod init;
pub mod install;
pub mod list;
pub mod search;
pub mod update;
pub mod upgrade;

// Re-export command functions
pub use add::run as run_add;
pub use delete::run as run_delete;
pub use info::run as run_info;
pub use init::run as run_init;
pub use install::run as run_install;
pub use list::run as run_list;
pub use search::run as run_search;
pub use update::run as run_update;
pub use upgrade::run as run_upgrade;

// Placeholders for future commands
// pub mod setup_path;
