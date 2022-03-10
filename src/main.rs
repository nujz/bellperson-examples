#![feature(option_zip)]

use std::ops::Mul;

use bellperson::{
    groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    },
    Circuit,
    SynthesisError::AssignmentMissing,
};
use blstrs::{Bls12, Scalar};
use rand::rngs::OsRng;

// f(x) = x^7 + x^5 + 3
// Prover: 我知道方程的值 f(x) = 138187310705323168727523 , 对应的一个解 x (2022).
struct MyCircuit {
    x: Option<Scalar>,
}

impl Circuit<Scalar> for MyCircuit {
    fn synthesize<CS: bellperson::ConstraintSystem<Scalar>>(
        self,
        cs: &mut CS,
    ) -> Result<(), bellperson::SynthesisError> {
        //
        // x * x = x^2
        // x^2 * x^2 = x^4
        // x^4 * x^2 = x^6
        // x^6 * x = x^7
        // x^4 * x = x^5
        // x^7 + x^5 + 3 = out <=> (x^7 + x^5 + 3) * 1 = out
        //

        let x = self.x;
        let x_var = cs.alloc(|| "x", || x.ok_or(AssignmentMissing))?;

        let x2 = x.map(|v| v.mul(v));
        let x2_var = cs.alloc(|| "x^2", || x2.ok_or(AssignmentMissing))?;

        cs.enforce(
            || "x * x = x^2",
            |lc| lc + x_var,
            |lc| lc + x_var,
            |lc| lc + x2_var,
        );

        let x4 = x2.map(|v| v.mul(v));
        let x4_var = cs.alloc(|| "x^4", || x4.ok_or(AssignmentMissing))?;

        cs.enforce(
            || "x^2 * x^2 = x^4",
            |lc| lc + x2_var,
            |lc| lc + x2_var,
            |lc| lc + x4_var,
        );

        let x6 = x4.zip_with(x2, |a, b| a.mul(b));
        let x6_var = cs.alloc(|| "x^6", || x6.ok_or(AssignmentMissing))?;

        cs.enforce(
            || "x^4 * x^2 = x^6",
            |lc| lc + x4_var,
            |lc| lc + x2_var,
            |lc| lc + x6_var,
        );

        let x7 = x6.zip_with(x, |a, b| a.mul(b));
        let x7_var = cs.alloc(|| "x^7", || x7.ok_or(AssignmentMissing))?;

        cs.enforce(
            || "x^6 * x = x^7",
            |lc| lc + x6_var,
            |lc| lc + x_var,
            |lc| lc + x7_var,
        );

        let x5 = x4.zip_with(x, |a, b| a.mul(b));
        let x5_var = cs.alloc(|| "x^5", || x5.ok_or(AssignmentMissing))?;

        cs.enforce(
            || "x^4 * x = x^5",
            |lc| lc + x4_var,
            |lc| lc + x_var,
            |lc| lc + x5_var,
        );

        let out = x7.zip_with(x5, |a, b| a + b + Scalar::from(3));


        // choose 1 or 2 ?
        //

        // 1:
        // 
        let out_var = cs.alloc_input(|| "out", || out.ok_or(AssignmentMissing))?;

        // 2: 
        //
        // let out_var = cs.alloc(|| "out", || out.ok_or(AssignmentMissing))?;
        // let out_input_var = cs.alloc_input(|| "out input", || out.ok_or(AssignmentMissing))?;
        // cs.enforce(
        //     || "input",
        //     |lc| lc + out_input_var,
        //     |lc| lc + CS::one(),
        //     |lc| lc + out_var,
        // );

        cs.enforce(
            || "(x^7 + x^5 + 3) * 1 = out",
            |lc| lc + x7_var + x5_var + (3.into(), CS::one()),
            |lc| lc + CS::one(),
            |lc| lc + out_var,
        );

        Ok(())
    }
}

fn main() {
    let rng = &mut OsRng::default();

    // Trust Setup
    let params = {
        let circuit = MyCircuit { x: None };
        generate_random_parameters::<Bls12, _, _>(circuit, rng).unwrap()
    };

    // Proofs Create
    let circuit = MyCircuit {
        x: Some(2022.into()),
    };
    let proof = create_random_proof(circuit, &params, rng).unwrap();
    // println!("{:?}", proof);

    // Proofs Verify
    let pvk = prepare_verifying_key(&params.vk);
    let is_valid = verify_proof(&pvk, &proof, &[fr_from_i128(138187310705323168727523)]).unwrap();
    println!("{:?}", is_valid);

    // false
    // let is_valid = verify_proof(&pvk, &proof, &[fr_from_i128(138187310705323168727522)]).unwrap();
    // println!("{:?}", is_valid);
}

fn fr_from_i128(v: i128) -> Scalar {
    let mut repr = [0u8; 32];
    repr[..16].copy_from_slice(&v.to_le_bytes());
    Scalar::from_bytes_le(&repr).unwrap()
}
