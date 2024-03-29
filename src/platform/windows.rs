use std::borrow::Cow;
use std::iter;

use enumset::EnumSet;
use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi;
use winapi::um::minwinbase::REASON_CONTEXT;
use winapi::um::winnt::{HANDLE, POWER_REQUEST_TYPE};
use winapi::um::{handleapi, winbase, winnt};

use crate::LockType;

#[derive(Debug)]
pub struct InhibitionManager();

impl InhibitionManager {
    pub fn new() -> Result<Self, Error> {
        Ok(Self())
    }
}

impl crate::InhibitionManager for InhibitionManager {
    type Error = Error;
    type Lock = Lock;

    fn lock(&self, types: EnumSet<LockType>, who: &str, why: &str) -> Result<Lock, Self::Error> {
        Lock::new(types, &format_msg(who, why))
    }
}

#[derive(Debug)]
pub enum Error {
    FailedToCreateRequest(DWORD),
    FailedToLock {
        lock_type: LockType,
        err_code: DWORD,
    },
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // TODO: format Windows error codes.
            Self::FailedToCreateRequest(_) => write!(f, "failed to create power request"),
            Self::FailedToLock { lock_type, .. } => {
                write!(f, "failed lock operation {:?}", lock_type)
            }
        }
    }
}

#[derive(Debug)]
pub struct Lock {
    request: PowerRequest,
    types: EnumSet<LockType>,
}

impl Lock {
    fn new(types: EnumSet<LockType>, msg: &str) -> Result<Self, Error> {
        let request = PowerRequest::new(msg).map_err(Error::FailedToCreateRequest)?;
        let mut failed: Option<(LockType, DWORD)> = None;
        for lock_type in types.iter() {
            let result =
                unsafe { winbase::PowerSetRequest(request.0, Self::request_type(lock_type)) };
            if result == 0 {
                failed = Some((lock_type, unsafe { errhandlingapi::GetLastError() }));
                break;
            }
        }
        match failed {
            Some((failed_type, err_code)) => {
                for lock_type in types.iter().take_while(|t| *t != failed_type) {
                    unsafe { winbase::PowerClearRequest(request.0, Self::request_type(lock_type)) };
                }
                Err(Error::FailedToLock {
                    lock_type: failed_type,
                    err_code,
                })
            }
            None => Ok(Self { request, types }),
        }
    }

    fn request_type(lock_type: LockType) -> POWER_REQUEST_TYPE {
        match lock_type {
            LockType::AutomaticSuspend => winnt::PowerRequestSystemRequired,
            LockType::ManualSuspend => winnt::PowerRequestAwayModeRequired,
            LockType::Screen => winnt::PowerRequestDisplayRequired,
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        for lock_type in self.types.iter() {
            unsafe { winbase::PowerClearRequest(self.request.0, Self::request_type(lock_type)) };
        }
    }
}

unsafe impl Send for Lock {}

impl crate::Lock for Lock {}

#[derive(Debug)]
struct PowerRequest(HANDLE);

impl PowerRequest {
    fn new(msg: &str) -> Result<Self, DWORD> {
        let mut context: REASON_CONTEXT = REASON_CONTEXT {
            Version: winnt::POWER_REQUEST_CONTEXT_VERSION,
            Flags: winnt::POWER_REQUEST_CONTEXT_SIMPLE_STRING,
            ..Default::default()
        };

        // Encode and null-terminate the string as a c-style utf16
        let mut text: Vec<u16> = msg.encode_utf16().chain(iter::once(0)).collect();
        unsafe { *context.Reason.SimpleReasonString_mut() = text.as_mut_ptr() };

        let request = unsafe { winbase::PowerCreateRequest(&mut context) };
        if request.is_null() {
            Err(unsafe { errhandlingapi::GetLastError() })
        } else {
            Ok(Self(request))
        }
    }
}

impl Drop for PowerRequest {
    fn drop(&mut self) {
        unsafe { handleapi::CloseHandle(self.0) };
    }
}

fn format_msg<'a>(who: &str, why: &'a str) -> Cow<'a, str> {
    if who.is_empty() {
        Cow::Borrowed(why)
    } else {
        Cow::Owned(format!("{}: {}", who, why))
    }
}
