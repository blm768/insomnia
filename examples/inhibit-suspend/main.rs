use std::time::Duration;

use insomnia::{platform, EnumSet, InhibitionManager, LockType};

fn main() -> Result<(), <platform::InhibitionManager as InhibitionManager>::Error> {
    let manager = insomnia::manager()?;

    let lock_a = manager.lock(
        EnumSet::only(LockType::AutomaticSuspend),
        "Suspend example",
        "testing automatic suspend inhibition",
    );
    eprintln!("{:?}", &lock_a);

    let lock_b = manager.lock(
        LockType::AutomaticSuspend | LockType::ManualSuspend,
        "Suspend example",
        "testing manual suspend inhibition",
    );
    eprintln!("{:?}", &lock_b);

    let lock_c = manager.lock(
        LockType::Screen.into(),
        "Suspend example",
        "testing screen sleep inhibition",
    );
    eprintln!("{:?}", &lock_c);

    std::thread::sleep(Duration::from_secs(5));
    Ok(())
}
