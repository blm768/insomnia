use std::time::Duration;

pub mod platform;

// TODO: need some way of handling disconnections.

pub trait InhibitionManager {
    type Error: std::error::Error;
    type Lock: Lock;

    // TODO: support locking multiple types at once, probably using EnumSet.
    fn lock(&self, lock_type: LockType, duration: LockDuration) -> Result<Self::Lock, Self::Error>;
}

pub trait Lock {
    fn duration(&self) -> LockDuration;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockType {
    AutomaticSuspend,
    ManualSuspend,
    ManualShutdown,
}

/// Describes how long a lock will be held
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockDuration {
    /// Inhibits power management until released
    Persistent,
    /// Inhibits power management only until the specified duration has elapsed
    Timed(Duration),
}

#[cfg(target_os = "linux")]
pub type PlatformManager = platform::linux::InhibitionManager;

// TODO: keep a single persistent instance? (probably...)
pub fn manager() -> Result<PlatformManager, <PlatformManager as InhibitionManager>::Error> {
    PlatformManager::new()
}
