#[cfg(target_os = "windows")]
use enum_map::Enum;
use enumset::{EnumSet, EnumSetType};

pub mod platform;

// TODO: need some way of handling disconnections.

pub trait InhibitionManager {
    type Error: std::error::Error;
    type Lock: Lock;

    fn lock(&self, types: EnumSet<LockType>) -> Result<Self::Lock, Self::Error>;
}

/// Inhibits a particular power management operation until the `Lock` is dropped.
///
/// Note that on some platforms, the lock *may* be terminated early under rare circumstances, i.e. if `systemd_logind` is restarted on Linux.
pub trait Lock {}

/* TODO: support inhibiting screensaver and monitor power saving?
(Requires using the GNOME API on Linux (need to investigate what APIs other DEs provide)
Probably requires using another API call on Windows to inhibit the screensaver; SetThreadExecutionState apparently doesn't do that. */
/// The type of power management operation to inhibit
///
/// Note that on some platforms, one variant of this enum may imply another. For instance, on Windows, it's not possible to inhibit `ManualSuspend` without also inhibiting `AutomaticSuspend`.
#[derive(Debug, EnumSetType)]
#[cfg_attr(target_os = "windows", derive(Enum))]
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
