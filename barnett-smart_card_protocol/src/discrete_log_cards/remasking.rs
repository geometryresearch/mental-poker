use crate::discrete_log_cards::{MaskedCard, Parameters, PublicKey};
use crate::{ComputationStatement, Provable, Verifiable};

use ark_ec::ProjectiveCurve;
use ark_ff::One;
use crypto_primitives::error::CryptoError;
use crypto_primitives::zkp::{proofs::chaum_pedersen_dl_equality, ArgumentOfKnowledge};
use std::ops::Mul;

pub type Statement<C> =
    ComputationStatement<MaskedCard<C>, MaskedCard<C>, (Parameters<C>, PublicKey<C>)>;

impl<C: ProjectiveCurve> Statement<C> {
    pub fn to_chaum_pedersen(
        &self,
    ) -> (
        chaum_pedersen_dl_equality::Parameters<C>,
        chaum_pedersen_dl_equality::Statement<C>,
    ) {
        // Unwrap the statement about cards
        let input_masked_card = self.input;
        let output_remasked_card = self.output;

        // Map to Chaum-Pedersen parameters
        let cp_parameters = chaum_pedersen_dl_equality::Parameters::new(
            self.public_parameters.0.enc_parameters.generator,
            self.public_parameters.1,
        );

        // Map to Chaum-Pedersen statement
        let minus_one = -C::ScalarField::one();
        let negative_original = input_masked_card.mul(minus_one);
        let statement_cipher = output_remasked_card + negative_original;
        let cp_statement =
            chaum_pedersen_dl_equality::Statement::new(statement_cipher.0, statement_cipher.1);

        (cp_parameters, cp_statement)
    }
}

pub struct Proof<C: ProjectiveCurve>(pub chaum_pedersen_dl_equality::proof::Proof<C>);

impl<C: ProjectiveCurve> Provable<chaum_pedersen_dl_equality::DLEquality<C>> for Statement<C> {
    type Output = Proof<C>;
    type Witness = C::ScalarField;

    fn prove(&self, witness: Self::Witness) -> Result<Self::Output, CryptoError> {
        let (cp_parameters, cp_statement) = self.to_chaum_pedersen();

        // Use witness to prove the statement
        let cp_proof =
            chaum_pedersen_dl_equality::DLEquality::prove(&cp_parameters, &cp_statement, &witness)?;

        Ok(Proof(cp_proof))
    }
}

impl<C: ProjectiveCurve> Verifiable<chaum_pedersen_dl_equality::DLEquality<C>> for Proof<C> {
    type Statement = Statement<C>;

    fn verify(&self, statement: &Self::Statement) -> Result<(), CryptoError> {
        let (cp_parameters, cp_statement) = statement.to_chaum_pedersen();

        chaum_pedersen_dl_equality::DLEquality::verify(&cp_parameters, &cp_statement, &self.0)
    }
}