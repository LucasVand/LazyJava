use std::sync::OnceLock;

pub static CONTEXT: OnceLock<GlobalContext> = OnceLock::new();

#[derive(Debug)]
pub struct GlobalContext {
    pub dry_run: bool,
}

impl GlobalContext {
    pub fn init(dry_run: bool) -> &'static GlobalContext {
        CONTEXT.get_or_init(|| GlobalContext { dry_run })
    }

    pub fn get() -> &'static GlobalContext {
        CONTEXT
            .get()
            .expect("GlobalContext has not been initialized")
    }

    pub fn is_dry_run() -> bool {
        CONTEXT.get().is_some_and(|ctx| ctx.dry_run)
    }
}
