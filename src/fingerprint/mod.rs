pub mod fingerprint;
pub mod hash;

// Re-export main functionality for easier access
pub use self::fingerprint::finger_print;
pub use self::hash::hash;
