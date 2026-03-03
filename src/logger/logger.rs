pub struct Logger {
    opts: LoggerOpts,
}

#[derive(Default)]
struct LoggerOpts {
    verbose: bool,
}

static mut LOG: Logger = Logger {
    opts: LoggerOpts { verbose: false },
};
impl Logger {
    pub fn verbose(value: bool) {
        unsafe {
            LOG.opts.verbose = value;
        }
    }

    pub fn log(msg: &str) {
        eprintln!("{}", msg);
    }
    pub fn elog(msg: &str) {
        eprintln!("{}", msg);
    }
    pub fn verbose_elog(msg: &str) {
        unsafe {
            if !LOG.opts.verbose {
                return;
            }
            eprintln!("{}", msg);
        }
    }
}
