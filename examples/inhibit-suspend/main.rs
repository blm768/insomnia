use insomnia::{InhibitionManager, LockType, PlatformManager};

fn main() -> Result<(), <PlatformManager as InhibitionManager>::Error> {
    let manager = insomnia::manager()?;
    eprintln!("{:?}", manager.lock(LockType::AutomaticSuspend));
    Ok(())
}
