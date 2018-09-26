#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate bellman;
extern crate pairing;
extern crate rand;
use pairing::{Engine, Field};
use bellman::{Circuit, ConstraintSystem, SynthesisError};

struct DemoCircuit<E: Engine> {
    a: Option<E::Fr>,
    b: Option<E::Fr>,
}

// Implementation of our circuit.
impl<'a, E: Engine> Circuit<E> for DemoCircuit<E> {
    fn synthesize<CS: ConstraintSystem<E>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let a = cs.alloc(|| "a", || {
            self.a.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let b = cs.alloc(|| "b", || {
            self.b.ok_or(SynthesisError::AssignmentMissing)
        })?;

        cs.enforce(|| "a is boolean", 
            |lc| lc + a,
            |lc| lc + a,
            |lc| lc + a,
        );

        cs.enforce(|| "b is boolean", 
            |lc| lc + b,
            |lc| lc + b,
            |lc| lc + b,
        );
            
        let c = cs.alloc_input(|| "c", || {
            match (self.a, self.b) {
                (Some(a), Some(b)) => {
                    if a.is_zero() == b.is_zero() {
                        Ok(E::Fr::zero())
                    } else {
                        Ok(E::Fr::one())
                    }
                }
                _ => Err(SynthesisError::AssignmentMissing)
            }
        })?;

        cs.enforce(|| "a ^ b = c",
            |lc| lc + a + a,
            |lc| lc + b,
            |lc| lc + a + b - c,
        );

        Ok(())
    }
}


// Create some parameters, create a proof, and verify the proof.
fn main() {
    use pairing::bls12_381::{Bls12, Fr};
    use std::marker::PhantomData;
    use rand::thread_rng;

    use bellman::groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof, Proof,
    };

    let rng = &mut thread_rng();

    let params = {
        let c = DemoCircuit::<Bls12> {
            a: None,
            b: None,
        };

        generate_random_parameters(c, rng).unwrap()
    };

    let pvk = prepare_verifying_key(&params.vk);

    let c = DemoCircuit::<Bls12> {
        a: Some(Fr::one()),
        b: Some(Fr::zero()),
    };

    // Create a groth16 proof with our parameters.
    let proof = create_random_proof(c, &params, rng).unwrap();

    let mut proof_data = vec![];

    proof.write(&mut proof_data).unwrap();

    let proof = Proof::read(&proof_data[..]).unwrap();

    assert!(verify_proof(&pvk, &proof, &[Fr::one()]).unwrap());
}
