use crate::args::LazyJavaGlobalArgs;

pub fn println_verbose(msg: &str, global: &LazyJavaGlobalArgs) {
    if !global.verbose {
        return;
    }
    println!("{}", msg);
}
