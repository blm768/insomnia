use insomnia::{InhibitionManager, LockDuration, LockType, PlatformManager};

fn main() -> Result<(), <PlatformManager as InhibitionManager>::Error> {
    let manager = insomnia::manager()?;
    eprintln!(
        "{:?}",
        manager.lock(LockType::AutomaticSuspend, LockDuration::Persistent)
    );
    Ok(())
}
