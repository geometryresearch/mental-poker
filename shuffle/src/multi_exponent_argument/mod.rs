use verifiable_threshold_masking_protocol::discrete_log_vtmp::{ElgamalCipher};
use ark_ec::{ProjectiveCurve};
use ark_ff::{Zero, PrimeField};
use std::marker::PhantomData;

pub mod error;
pub mod prover;
pub mod proof;

pub struct Parameters<'a, C: ProjectiveCurve> {
    pub public_key: &'a C::Affine,
    pub commit_key: &'a Vec<C::Affine>,
    pub masking_generator: C::Affine,
}

impl<'a, C: ProjectiveCurve> Parameters<'a, C> {
    pub fn new(public_key: &'a C::Affine, commit_key: &'a Vec<C::Affine>, masking_generator: C::Affine) -> Self {
        Self {
            public_key, 
            commit_key,
            masking_generator
        }
    }
}

pub struct Witness<'a, C: ProjectiveCurve> {
    pub matrix_a: &'a Vec<Vec<C::ScalarField>>,
    pub matrix_blinders: &'a Vec<C::ScalarField>,
    pub ro: C::ScalarField
}

impl<'a, C: ProjectiveCurve> Witness<'a, C> {
    pub fn new(matrix_a: &'a Vec<Vec<C::ScalarField>>, matrix_blinders: &'a Vec<C::ScalarField>, ro: C::ScalarField) -> Self {
        Self {
            matrix_a, 
            matrix_blinders, 
            ro
        }
    }
}

pub struct Statement<'a, C: ProjectiveCurve> {
    pub shuffled_ciphers: &'a Vec<Vec<ElgamalCipher<C>>>,
    pub product: ElgamalCipher<C>,
    pub commitments_to_exponents: &'a Vec<C>
}

impl<'a, C: ProjectiveCurve> Statement<'a, C> {
    pub fn new(shuffled_ciphers: &'a Vec<Vec<ElgamalCipher<C>>>, product: ElgamalCipher<C>, commitments_to_exponents: &'a Vec<C>) -> Self {
        Self {
            shuffled_ciphers, 
            product, 
            commitments_to_exponents
        }
    }
}

use error::Error;

pub fn diagonals_from_chunks_for_testing<C: ProjectiveCurve>(
    cipher_chunks: &Vec<Vec<ElgamalCipher<C>>>,
    scalar_chunks: &Vec<Vec<C::ScalarField>>,
    claimed_product: ElgamalCipher<C>,
    a_0_randomness: &Vec<C::ScalarField>,
)
 -> Result<Vec<ElgamalCipher<C>>, Error> {

    let m = cipher_chunks.len();
    let num_of_diagonals = 2 * m - 1;

    let mut diagonal_sums: Vec<ElgamalCipher<C>> = vec![ElgamalCipher::zero(); num_of_diagonals];
    let center = num_of_diagonals/2 as usize;

    for d in 1..m {
        let additional_randomness = DotProductCalculator::<C>::scalars_by_ciphers(&a_0_randomness, &cipher_chunks[d-1]).unwrap();
        let mut tmp_product1 = ElgamalCipher::zero(); 
        let mut tmp_product2 = ElgamalCipher::zero(); 
        for i in d..m {
            let dot = DotProductCalculator::<C>::scalars_by_ciphers(&scalar_chunks[i - d], &cipher_chunks[i]).unwrap();
            tmp_product1 = tmp_product1 + dot;

            let dot = DotProductCalculator::<C>::scalars_by_ciphers(&scalar_chunks[i], &cipher_chunks[i - d]).unwrap();
            tmp_product2 = tmp_product2 + dot;
        }

        diagonal_sums[center - d] = tmp_product1 + additional_randomness;
        diagonal_sums[center + d] = tmp_product2;
    }
    
    diagonal_sums[center] = claimed_product;

    let zeroth_diagonal = DotProductCalculator::<C>::scalars_by_ciphers(&a_0_randomness, &cipher_chunks.last().unwrap()).unwrap();
    diagonal_sums.insert(0, zeroth_diagonal);

    Ok(diagonal_sums)  
}

pub trait DotProduct<C:ProjectiveCurve> {
    type Scalar;
    type Point;
    type Ciphertext;

    fn scalars_by_ciphers(
        scalars: &Vec<Self::Scalar>,
        ciphers: &Vec<Self::Ciphertext>)
        -> Result<Self::Ciphertext, Error>;

    fn scalars_by_points(
        scalars: &Vec<Self::Scalar>,
        points: &Vec<Self::Point>)
        -> Result<Self::Point, Error>;

    fn scalars_by_scalars(
        scalars_a: &Vec<Self::Scalar>,
        scalar_b: &Vec<Self::Scalar>)
        -> Result<Self::Scalar, Error>;
}

pub struct DotProductCalculator<C: ProjectiveCurve> {
    _curve: PhantomData<C>
}

impl<C: ProjectiveCurve> DotProduct<C> for DotProductCalculator<C> {
    type Scalar = C::ScalarField;
    type Point = C;
    type Ciphertext = ElgamalCipher<C>;

    fn scalars_by_ciphers(
        scalars: &Vec<Self::Scalar>,
        ciphers: &Vec<Self::Ciphertext>)
        -> Result<Self::Ciphertext, Error> {
        
        if ciphers.len() != scalars.len() {
            return Err(Error::DotProductLenError)
        }
    
        let dot_product: Self::Ciphertext = ciphers.iter().zip(scalars.iter()).map(|(cipher, scalar)| *cipher * *scalar).sum();
    
        Ok(dot_product)
    }


    fn scalars_by_points(
        scalars: &Vec<Self::Scalar>,
        points: &Vec<Self::Point>)
        -> Result<Self::Point, Error> {
        
        if points.len() != scalars.len() {
            return Err(Error::DotProductLenError)
        }
    
        let dot_product: Self::Point = points.iter().zip(scalars.iter()).map(|(&point, scalar)| point.mul(scalar.into_repr())).sum();
    
        Ok(dot_product)
    }
    
    fn scalars_by_scalars(
        scalars_a: &Vec<Self::Scalar>,
        scalars_b: &Vec<Self::Scalar>)
        -> Result<Self::Scalar, Error> {
        
        if scalars_a.len() != scalars_b.len() {
            return Err(Error::DotProductLenError)
        }
    
        let dot_product: Self::Scalar = scalars_a.iter().zip(scalars_b.iter()).map(|(s_a, s_b)| *s_a * *s_b).sum();
    
        Ok(dot_product)
    }
}


#[cfg(test)]
mod test {

    use starknet_curve::{Projective, Fr};
    use ark_std::{rand::{thread_rng}, UniformRand};
    use verifiable_threshold_masking_protocol::discrete_log_vtmp::{ElgamalCipher};
    use super::{DotProduct, DotProductCalculator, diagonals_from_chunks_for_testing};
    use ark_ff::{Zero};

    #[test]
    fn test_dot_product() {
        let rng = &mut thread_rng();
        
        let c1 = ElgamalCipher::<Projective>(Projective::rand(rng).into(), Projective::rand(rng).into());
        let c2 = ElgamalCipher::<Projective>(Projective::rand(rng).into(), Projective::rand(rng).into());
        let c3 = ElgamalCipher::<Projective>(Projective::rand(rng).into(), Projective::rand(rng).into());


        let c = vec![c1, c2, c3];

        let scalars = vec![Fr::rand(rng), Fr::rand(rng), Fr::rand(rng)];

        let manual_dot_product = c1 * scalars[0] + c2 * scalars[1] + c3 * scalars[2];

        assert_eq!(manual_dot_product, DotProductCalculator::scalars_by_ciphers(&scalars, &c).unwrap());
    }

    #[test]
    fn test_diagonal_sums() {
        let m = 3;
        let n = 2;
        let rng = &mut thread_rng();

        let number_of_ciphers = m*n;

        let mut ciphers: Vec<ElgamalCipher<Projective>> = vec![ElgamalCipher::zero(); number_of_ciphers];
        let mut scalars: Vec<Fr> = vec![Fr::zero(); number_of_ciphers];
        let random: Vec<Fr> = vec![Fr::rand(rng); n];

        let test_cipher = ElgamalCipher::<Projective>(Projective::rand(rng).into(), Projective::rand(rng).into());

        for i in 0..number_of_ciphers {
            ciphers[i] = test_cipher.clone();
            scalars[i] = Fr::rand(rng);
        }

        let cipher_chunks = ciphers.chunks(n).map(|c| c.to_vec()).collect::<Vec<_>>();
        let scalars_chunks = scalars.chunks(n).map(|c| c.to_vec()).collect::<Vec<_>>();

        let product = DotProductCalculator::<Projective>::scalars_by_ciphers(&scalars, &ciphers).unwrap();
        
        let result = diagonals_from_chunks_for_testing::<Projective>(&cipher_chunks, &scalars_chunks, product, &random).unwrap();

        let manual_result = vec![
            test_cipher * (random[0] + random[1]), 
            test_cipher * (random[0] + random[1] + scalars[0] + scalars[1]),
            test_cipher * (random[0] + random[1] + scalars[0] + scalars[1] + scalars[2] + scalars[3]),
            test_cipher * (scalars[0] + scalars[1] + scalars[2] + scalars[3] + scalars[4] + scalars[5]),
            test_cipher * (scalars[2] + scalars[3] + scalars[4] + scalars[5]),
            test_cipher * (scalars[4] + scalars[5])
        ];

        assert_eq!(result, manual_result);
    }
}