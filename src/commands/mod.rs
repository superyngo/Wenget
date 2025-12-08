//! Command implementations for WenPM

pub mod add;
pub mod bucket;
pub mod delete;
pub mod info;
pub mod init;
pub mod list;
pub mod repair;
pub mod search;
pub mod update;

// Re-export command functions
pub use add::run as run_add;
pub use bucket::run as run_bucket;
pub use delete::run as run_delete;
pub use info::run as run_info;
pub use init::run as run_init;
pub use list::run as run_list;
pub use repair::run as run_repair;
pub use search::run as run_search;
pub use update::run as run_update;

// Placeholders for future commands
// pub mod setup_path;
