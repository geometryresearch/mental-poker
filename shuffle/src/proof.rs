use ark_ec::ProjectiveCurve;
use ark_ff::{Field, Zero, PrimeField};

use merlin::Transcript;
use crate::transcript::TranscriptProtocol;
use crate::product_argument::single_value_product_argument::proof::{Proof as ProductArgumentProof};
use crate::utils::{HomomorphicCommitment, PedersenCommitment};

use crate::config::PublicConfig;
use crate::error::Error;

use std::iter;


pub struct Proof<C, const SIZE: usize> 
where 
    C: ProjectiveCurve
{
    pub (crate) pi_commit: C,
    pub (crate) exp_pi_commit: C,
    pub (crate) product_argument_proof: ProductArgumentProof<C, SIZE>
}

impl<C, const SIZE: usize> Proof<C, SIZE> 
    where
        C: ProjectiveCurve
{
    pub fn verify(&self, 
        config: &PublicConfig<C, SIZE>, 
        transcript: &mut Transcript,
    ) -> Result<(), Error> {

        let mut transcript = transcript.clone();
        transcript.append(b"commit_key", &config.commit_key);

        transcript.append(b"pi_commit", &self.pi_commit);
        let x: C::ScalarField = transcript.challenge_scalar(b"x");

        transcript.append(b"exp_pi_commit", &self.exp_pi_commit);

        let y: C::ScalarField = transcript.challenge_scalar(b"y");
        let z: C::ScalarField = transcript.challenge_scalar(b"z");

        let mut identity_permutation: Vec<u64> = Vec::with_capacity(SIZE);
        for i in 0..SIZE {
            identity_permutation.push(i as u64);
        }

        let zero = C::ScalarField::zero();
        let z_arr = iter::repeat(-z).take(SIZE).collect();
        let z_commit = PedersenCommitment::<C>::commit_vector(&config.commit_key, &z_arr, zero);

        let d_commit = self.pi_commit.mul(y.into_repr()) + self.exp_pi_commit;
        let d_minus_z_commit = d_commit + z_commit;

        let b: C::ScalarField = identity_permutation.iter()
                .map(|i| C::ScalarField::from(*i)*y + x.pow(Self::as_limbs(*i)) - z)
                .collect::<Vec<_>>()
                .iter()
                .product();

        transcript.append(b"b", &b);

        assert_eq!(self.product_argument_proof.verify(&config, b, d_minus_z_commit, &mut transcript), Ok(()));

        Ok(())
    }

    fn as_limbs(p_i: u64) -> [u64; 4] {
        [p_i, 0, 0, 0]
    }
}