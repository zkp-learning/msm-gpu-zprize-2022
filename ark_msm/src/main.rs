use ark_bls12_381::{Fr, G1Affine, G1Projective};
use ark_ec::scalar_mul::variable_base::VariableBaseMSM;
use ark_ec::AffineRepr;
use ark_ec::Group;
use ark_ff::BigInteger;
use ark_std::iterable::Iterable;
use ark_std::UniformRand;
use ark_std::Zero;
use std::env;
use std::io::{BufReader, BufWriter};
use std::ops::Mul;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{convert::TryInto, fs::File};
use tracing::{span, Level};
use tracing_flame::FlameLayer;
use tracing_subscriber::{fmt, prelude::*, registry::Registry};

const N: usize = 1 << 20;
static PATH: &str = "flame.folded";

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
    let start = Instant::now();
    let base = generate_base();
    let duration = start.elapsed();
    println!("generate_base executed in: {:?}", duration);

    let start = Instant::now();
    let fr = generate_random_fr();
    let duration_fr = start.elapsed();
    println!("generate_random_fr executed in: {:?}", duration_fr);

    let start = Instant::now();
    let result_ark = G1Projective::msm(&base, &fr).unwrap();
    let duration_msm = start.elapsed();
    println!("ark msm executed in: {:?}", duration_msm);

    let start = Instant::now();
    let result_naive = naive_msm(&base, &fr);
    let duration_msm = start.elapsed();
    println!("naive msm executed in: {:?}", duration_msm);

    let start = Instant::now();
    let result_bit = msm(&base, &fr);
    let duration_msm = start.elapsed();
    println!("bit-op msm executed in: {:?}", duration_msm);

    assert_eq!(result_ark, result_naive);
    assert_eq!(result_ark, result_bit);

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

    // span!(Level::INFO, "outer").in_scope(|| {

    //     span!(Level::INFO, "Inner").in_scope(|| {

    //         span!(Level::INFO, "Innermost").in_scope(|| {

    //         });
    //     });
    // });

    // drop the guard to make sure the layer flushes its output then read the
    // output to create the flamegraph
    drop(guard);
    make_flamegraph(tmp_dir.path(), out.as_ref());
}

fn generate_base() -> Vec<G1Affine> {
    let mut rng = ark_std::test_rng();
    let mut base = [G1Affine::zero(); N];
    for i in 0..N {
        base[i] = G1Affine::rand(&mut rng);
    }
    base
}

fn generate_random_fr() -> Vec<Fr> {
    let mut rng = ark_std::test_rng();
    let mut random_fr = [Fr::zero(); N];
    for i in 0..N {
        random_fr[i] = Fr::rand(&mut rng);
    }
    random_fr
}

fn naive_msm(base: &[G1Affine], fr: &[Fr]) -> G1Projective {
    let mut result = G1Projective::zero();
    for i in 0..N {
        result += &base[i].mul(fr[i]);
    }
    result
}

fn msm(base: &[G1Affine], fr: &[Fr]) -> G1Projective {
    let mut res = G1Projective::zero();

    for (base, fr) in base.iter().zip(fr) {
        let mut base_item = G1Projective::from(base);
        let fr_item = fr_to_bits(fr);

        for i in 0..256 {
            if fr_item[i] {
                res += base_item;
            }
            base_item.double_in_place();
        }
    }
    G1Projective::msm(base, fr).unwrap()
}

fn fr_to_bits(fr: &Fr) -> Vec<bool> {
    let mut bits = vec![];
    let bytes = fr.0.to_bytes_be();

    for byte in bytes.iter() {
        for i in 0..8 {
            let bit = (byte >> i) & 1 == 1;
            bits.push(bit);
        }
    }
    bits.try_into().unwrap()
}
