use enumset::{EnumSet, EnumSetType};

pub mod platform;

// TODO: need some way of handling disconnections.

pub trait InhibitionManager {
    type Error: std::error::Error;
    type Lock: Lock;

    fn lock(&self, types: EnumSet<LockType>) -> Result<Self::Lock, Self::Error>;
}

pub trait Lock {}

/* TODO: support inhibiting screensaver and monitor power saving?
(Requires using the GNOME API on Linux (need to investigate what APIs other DEs provide)
Probably requires using another API call on Windows to inhibit the screensaver; SetThreadExecutionState apparently doesn't do that. */
#[derive(Debug, EnumSetType)]
pub enum LockType {
    AutomaticSuspend,
    ManualSuspend,
    // TODO: put this behind a feature instead? (not supported on Windows.)
    #[cfg(target_os = "linux")]
    ManualShutdown,
}

// TODO: keep a single persistent instance? (probably...)
pub fn manager(
) -> Result<platform::InhibitionManager, <platform::InhibitionManager as InhibitionManager>::Error>
{
    platform::InhibitionManager::new()
}
