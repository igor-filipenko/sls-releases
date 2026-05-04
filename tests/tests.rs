mod logging;

#[ctor::ctor]
fn init_tracing_before_tests() {
    logging::init();
}

#[path = "routes/mod.rs"]
mod routes;
#[path = "jobs/sync.rs"]
mod sync;
