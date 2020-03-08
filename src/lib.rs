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

/* TODO: support inhibiting screensaver and monitor power saving?
(Requires using the GNOME API on Linux (need to investigate what APIs other DEs provide)
Probably requires using another API call on Windows to inhibit the screensaver; SetThreadExecutionState apparently doesn't do that. */
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockType {
    AutomaticSuspend,
    ManualSuspend,
    ManualShutdown,
}

/// Describes how long a lock will be held
/* TODO: consider getting rid of this. The GNOME API doesn't support non-persistent locks,
the systemd API only supports them for sleep and shutdown, and the Windows API just supports
resetting the idle timer. None of these APIs supports an explicit user-provided duration, so
we'd have to implement that ourselves anyway, and that may be out of scope for us.*/
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
