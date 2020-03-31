use std::borrow::Cow;
use std::mem::ManuallyDrop;

use dbus::arg::OwnedFd;
use enumset::EnumSet;
use logind_dbus::{LoginManager, LoginManagerConnection};

use crate::LockType;

pub struct InhibitionManager {
    _manager: Box<LoginManager>,
    connection: ManuallyDrop<LoginManagerConnection<'static>>,
}

impl InhibitionManager {
    pub fn new() -> Result<Self, dbus::Error> {
        let manager = Box::new(LoginManager::new()?);
        let static_manager: &'static _ = unsafe { &*(&*manager as *const LoginManager) };
        let connection = ManuallyDrop::new(static_manager.connect());
        Ok(Self {
            _manager: manager,
            connection,
        })
    }
}

impl Drop for InhibitionManager {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.connection) };
    }
}

impl crate::InhibitionManager for InhibitionManager {
    type Error = dbus::Error;
    type Lock = Lock;

    fn lock(&self, types: EnumSet<LockType>) -> Result<Lock, Self::Error> {
        // TODO: provide better where/why info.
        let types_str = lock_types_str(types);
        // TODO: recover from connection closure?
        let handle = self.connection.inhibit(&types_str, "who", "why", "block")?;
        Ok(Lock { _handle: handle })
    }
}

#[derive(Debug)]
pub struct Lock {
    _handle: OwnedFd,
}

impl crate::Lock for Lock {}

fn lock_types_str(types: EnumSet<LockType>) -> Cow<'static, str> {
    match types.len() {
        0 => Cow::Borrowed(""),
        1 => Cow::Borrowed(lock_type_name(types.iter().next().unwrap())),
        _ => {
            let names: Vec<_> = types.iter().map(lock_type_name).collect();
            Cow::Owned(names[..].join(":"))
        }
    }
}

fn lock_type_name(lock_type: LockType) -> &'static str {
    match lock_type {
        LockType::AutomaticSuspend => "idle",
        LockType::ManualSuspend => "sleep",
        LockType::ManualShutdown => "shutdown",
    }
}
