#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::{FftField, Field, One, UniformRand};
    use ark_poly::{polynomial::univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
    use ark_r1cs_std::{
        alloc::AllocVar,
        fields::{fp::FpVar, FieldVar},
        poly::{domain::Radix2DomainVar, evaluations::univariate::EvaluationsVar},
        R1CSVar,
    };
    use ark_relations::ns;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_std::test_rng;

    /// The following test is directly from
    /// https://github.com/arkworks-rs/r1cs-std/tree/b477880a3b9c097b65b1e006d8116f3998cf013c/src/poly/evaluations/univariate/mod.rs#L380
    /// with the only change of adapting it to depend on 'n' instead of hardcoded values, and
    /// adding some 'dbg!' prints.
    ///
    /// run the test:
    /// > cargo test -- --nocapture
    #[test]
    fn test_interpolate_constant_offset() {
        let mut rng = test_rng();

        // For n<=11 test will pass, for n>11 test will get stack overflow
        let n = 12;
        dbg!(n);

        let poly = DensePolynomial::rand((1 << n) - 1, &mut rng);
        let gen = Fr::get_root_of_unity(1 << n).unwrap();
        assert_eq!(gen.pow(&[1 << n]), Fr::one());
        let domain = Radix2DomainVar::new(gen, n, FpVar::constant(Fr::rand(&mut rng))).unwrap();
        let mut coset_point = domain.offset().value().unwrap();
        let mut oracle_evals = Vec::new();
        for _ in 0..(1 << n) {
            oracle_evals.push(poly.evaluate(&coset_point));
            coset_point *= gen;
        }
        let cs = ConstraintSystem::new_ref();
        let evaluations_fp: Vec<_> = oracle_evals
            .iter()
            .map(|x| FpVar::new_input(ns!(cs, "evaluations"), || Ok(x)).unwrap())
            .collect();
        let evaluations_var = EvaluationsVar::from_vec_and_domain(evaluations_fp, domain, true);

        let interpolate_point = Fr::rand(&mut rng);
        let interpolate_point_fp =
            FpVar::new_input(ns!(cs, "interpolate point"), || Ok(interpolate_point)).unwrap();

        let expected = poly.evaluate(&interpolate_point);

        let actual = evaluations_var
            .interpolate_and_evaluate(&interpolate_point_fp)
            .unwrap()
            .value()
            .unwrap();

        assert_eq!(actual, expected);
        assert!(cs.is_satisfied().unwrap());
        println!("number of constraints: {}", cs.num_constraints())
    }
}
