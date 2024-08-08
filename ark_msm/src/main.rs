use ark_bls12_381::{Fr, G1Affine, G1Projective};
use ark_ec::scalar_mul::variable_base::VariableBaseMSM;
use ark_ec::AffineRepr;
use ark_ec::Group;
use ark_ff::BigInteger;
use ark_std::iterable::Iterable;
use ark_std::UniformRand;
use ark_std::Zero;
use std::ops::Mul;
use std::time::Instant;

const N: usize = 16384;

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
}

fn generate_base() -> [G1Affine; N] {
    let mut rng = ark_std::test_rng();
    let mut base = [G1Affine::zero(); N];
    for i in 0..N {
        base[i] = G1Affine::rand(&mut rng);
    }
    base
}

fn generate_random_fr() -> [Fr; N] {
    let mut rng = ark_std::test_rng();
    let mut random_fr = [Fr::zero(); N];
    for i in 0..N {
        random_fr[i] = Fr::rand(&mut rng);
    }
    random_fr
}

fn naive_msm(base: &[G1Affine; N], fr: &[Fr; N]) -> G1Projective {
    let mut result = G1Projective::zero();
    for i in 0..N {
        result += &base[i].mul(fr[i]);
    }
    result
}

fn msm(base: &[G1Affine; N], fr: &[Fr; N]) -> G1Projective {
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

fn fr_to_bits(fr: &Fr) -> [bool; 256] {
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
