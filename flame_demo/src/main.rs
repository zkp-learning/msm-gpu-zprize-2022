use std::{
    convert::TryInto,
    env,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tracing::{span, Level};
use tracing_flame::FlameLayer;
use tracing_subscriber::{fmt, prelude::*, registry::Registry};

fn setup_global_collector(dir: &Path) -> impl Drop {
    let fmt_layer = fmt::Layer::default();

    let (flame_layer, _guard) = FlameLayer::with_file(dir.join(PATH)).unwrap();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(flame_layer)
        .init();
    _guard
}

fn make_flamegraph(tmpdir: &Path, out: &Path) {
    println!("outputting flamegraph to {}", out.display());
    let inf = File::open(tmpdir.join(PATH)).unwrap();
    let reader = BufReader::new(inf);

    let out = File::create(out).unwrap();
    let writer = BufWriter::new(out);

    let mut opts = inferno::flamegraph::Options::default();
    inferno::flamegraph::from_reader(&mut opts, reader, writer).unwrap();
}

fn main() {
    let out = if let Some(arg) = env::args().nth(1) {
        PathBuf::from(arg)
    } else {
        let mut path = env::current_dir().expect("failed to read current directory");
        path.push("tracing-flame-inferno.svg");
        path
    };

    let tmp_dir = tempfile::Builder::new()
        .prefix("flamegraphs")
        .tempdir()
        .expect("failed to create temporary directory");
    let guard = setup_global_collector(tmp_dir.path());

    span!(Level::INFO, "outer").in_scope(|| {
        span!(Level::INFO, "Inner").in_scope(|| {
            span!(Level::INFO, "Innermost").in_scope(|| {});
        });
    });

    // drop the guard to make sure the layer flushes its output then read the
    // output to create the flamegraph
    drop(guard);
    make_flamegraph(tmp_dir.path(), out.as_ref());
}
