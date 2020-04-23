use std::borrow::Cow;
use std::mem::ManuallyDrop;

use dbus::arg::OwnedFd;
use enumset::EnumSet;
use logind_dbus::{LoginManager, LoginManagerConnection};

use crate::LockType;

pub struct InhibitionManager {
    manager: Box<LoginManager>,
    connection: Option<ManuallyDrop<LoginManagerConnection<'static>>>,
}

impl InhibitionManager {
    pub fn new() -> Result<Self, dbus::Error> {
        let manager = Box::new(LoginManager::new()?);
        let static_manager: &'static _ = unsafe { &*(&*manager as *const LoginManager) };
        let connection = Some(ManuallyDrop::new(static_manager.connect()));
        Ok(Self {
            manager,
            connection,
        })
    }

    fn reconnect(&mut self) -> &LoginManagerConnection {
        // TODO: can we detect and handle disconnections from the dbus daemon itself?
        let static_manager: &'static _ = unsafe { &*(&*self.manager as *const LoginManager) };
        let new_conn = static_manager.connect();
        let _ = self
            .connection
            .replace(ManuallyDrop::new(new_conn))
            .map(ManuallyDrop::into_inner);
        self.connection.as_ref().unwrap()
    }

    fn disconnect(&mut self) {
        let _ = self.connection.take().map(ManuallyDrop::into_inner);
    }
}

impl Drop for InhibitionManager {
    fn drop(&mut self) {
        self.disconnect();
    }
}

impl crate::InhibitionManager for InhibitionManager {
    type Error = dbus::Error;
    type Lock = Lock;

    fn lock(
        &mut self,
        types: EnumSet<LockType>,
        who: &str,
        why: &str,
    ) -> Result<Lock, Self::Error> {
        let types_str = lock_types_str(types);
        let conn = match self.connection {
            Some(ref c) => c as &LoginManagerConnection<'_>,
            None => self.reconnect(),
        };
        let result = conn.inhibit(&types_str, who, why, "block");
        match result {
            Ok(handle) => Ok(Lock { _handle: handle }),
            Err(e) => {
                if treat_as_disconnect(&e) {
                    self.disconnect();
                }
                Err(e)
            }
        }
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

fn treat_as_disconnect(err: &dbus::Error) -> bool {
    match err.name() {
        Some("org.freedesktop.DBus.Error.NoReply") => true,
        Some("org.freedesktop.DBus.Error.Timeout") => true,
        Some("org.freedesktop.DBus.Error.Disconnected") => true,
        _ => false,
    }
}
