use ark_bls12_381::{Fr, G1Affine, G1Projective};
use ark_ec::scalar_mul::variable_base::VariableBaseMSM;
use ark_ec::Group;
use ark_ff::BigInteger;
use ark_ff::PrimeField;
use ark_std::UniformRand;
use ark_std::Zero;

const C: usize = 15;
const W: usize = 18;

// pub fn pippenger_msm(base: &[G1Affine], fr: &[Fr]) -> G1Projective {
//     let

// }

// fn w_j()

fn fr_to_c_bit(fr: &Fr) -> [Fr; 18] {
    let mut res = [Fr::from(0); 18];

    let mut bits = Vec::with_capacity(256);
    let bytes = fr.into_bigint().to_bytes_le();

    for byte in bytes.iter() {
        for i in 0..8 {
            let bit = ((byte >> i) & 1) == 1;
            bits.push(bit);
        }
    }
    for i in 0..17 {
        let start_idx = C * i;
        let slice = &bits[start_idx..(start_idx + 15)];
        res[i] = bit_to_fr(slice);
    }
    res[17] = bit_to_fr(&bits[255..]);

    res
}

pub fn bit_to_fr(vals: &[bool]) -> Fr {
    let mut res: u64 = 0;
    for (i, val) in vals.iter().enumerate() {
        if *val {
            res += 1 << i;
        }
    }
    Fr::from(res)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::fr_to_bits;

    #[test]
    fn test_fr_to_c_bit() {
        let fr = Fr::from(8u64 + 2u64.pow(15));
        let res = fr_to_c_bit(&fr);
        for i in 0..18 {
            println!("{:?}", fr_to_bits(&res[i]));
        }
    }
}
