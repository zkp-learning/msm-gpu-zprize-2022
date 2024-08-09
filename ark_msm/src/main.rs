use ark_bls12_381::{Fr, G1Affine, G1Projective};
use ark_ec::scalar_mul::variable_base::VariableBaseMSM;
use ark_ec::Group;
use ark_ff::BigInteger;
use ark_ff::PrimeField;
use ark_std::UniformRand;
use ark_std::Zero;
use rayon::prelude::*;
use std::ops::Mul;
use std::time::Instant;

const N: usize = 1 << 19;

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

fn generate_base() -> Vec<G1Affine> {
    let mut res = Vec::with_capacity(N);
    (0..N)
        .into_par_iter()
        .map(|_| {
            let mut rng = ark_std::test_rng();

            G1Affine::rand(&mut rng)
        })
        .collect_into_vec(&mut res);
    res
}

fn generate_random_fr() -> Vec<Fr> {
    let mut rng = ark_std::test_rng();
    let mut random_fr = Vec::with_capacity(N);
    for _ in 0..N {
        random_fr.push(Fr::rand(&mut rng));
    }
    random_fr
}

fn naive_msm(base: &[G1Affine], fr: &[Fr]) -> G1Projective {
    base.par_iter().enumerate().map(|(i, b)| b.mul(fr[i])).sum()
}

fn msm(bases: &[G1Affine], fr: &[Fr]) -> G1Projective {
    bases
        .par_iter()
        .enumerate()
        .map(|(i, base)| {
            let mut base_item = G1Projective::from(*base);
            let mut sum = G1Projective::zero();
            let fr_item = fr_to_bits(&fr[i]);

            for i in 0..256 {
                if fr_item[i] {
                    sum += base_item;
                }
                base_item.double_in_place();
            }
            sum
        })
        .sum()
}

fn fr_to_bits(fr: &Fr) -> Vec<bool> {
    let mut bits = Vec::with_capacity(256);
    let bytes = fr.into_bigint().to_bytes_le();

    for byte in bytes.iter() {
        for i in 0..8 {
            let bit = ((byte >> i) & 1) == 1;
            bits.push(bit);
        }
    }
    bits
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fr_to_bits() {
        let fr = Fr::from(8u64);

        let bits = fr_to_bits(&fr);

        println!("{:?}", bits);
    }
}
