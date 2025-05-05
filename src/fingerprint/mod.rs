pub mod fingerprint;
pub mod hash;
pub mod peaks;
pub mod spectogram;
pub mod utils;
// Re-export main functionality for easier access
pub use self::fingerprint::finger_print;
pub use self::hash::hash;
pub use self::utils::frame_signal;
