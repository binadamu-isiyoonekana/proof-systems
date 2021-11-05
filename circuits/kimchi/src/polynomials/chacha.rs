//! This source file implements the cha-cha constraints, which are rather simple aside form the lookups.
//!
//! There are four chacha constraint types, corresponding to the four lines in each quarter round.
//!
//! <pre>
//! a += b; d ^= a; d <<<= 16;
//! c += d; b ^= c; b <<<= 12;
//! a += b; d ^= a; d <<<= 8;
//! c += d; b ^= c; b <<<= 7;
//! </pre>
//!
//! or, written without mutation, (and where + is mod 2**32),
//!
//! <pre>
//! a'  = a + b ; d' = (d ^ a') <<< 16;
//! c'  = c + d'; b' = (b ^ c') <<< 12;
//! a'' = a' + b'; d'' = (d' ^ a') <<< 8;
//! c'' = c' + d''; b'' = (c'' ^ b') <<< 7;
//! </pre>
//!
//! We lay each line as two rows.
//!
//! Each line has the form
//!
//! <pre>
//! x += z; y ^= x; y <<<= k
//! </pre>
//!
//! or without mutation,
//!
//! <pre>
//! x' = x + z; y' = (y ^ x') <<< k
//! </pre>
//!
//! which we abbreviate as
//!
//! <pre>
//! L(x, x', y, y', z, k).
//! </pre>
//!
//! In general, such a line will be laid out as the two rows
//!
//! <pre>
//! x  y  z       (y^x')_0 (y^x')_1 (y^x')_2 (y^x')_3 (x+z)_0 (x+z)_1 (x+z)_2 (x+z)_3 y_0 y_1 y_2 y_3
//! x' y' (x+z)_8 (y^x')_4 (y^x')_5 (y^x')_6 (y^x')_7 (x+z)_4 (x+z)_5 (x+z)_6 (x+z)_7 y_4 y_5 y_6 y_7
//! </pre>
//!
//! where A_i indicates the i^th nybble (four-bit chunk) of the value A.
//!
//! (x+z)_8 is special, since we know it is actually at most 1 bit (representing the overflow bit of x + z).
//!
//! So the first line L(a, a', d, d', b, 8) for example becomes the two rows
//!
//! <pre>
//! a  d  b       (d^a')_0 (d^a')_1 (d^a')_2 (d^a')_3 (a+b)_0 (a+b)_1 (a+b)_2 (a+b)_3 d_0 d_1 d_2 d_3
//! a' d' (a+b)_8 (d^a')_4 (d^a')_5 (d^a')_6 (d^a')_7 (a+b)_4 (a+b)_5 (a+b)_6 (a+b)_7 d_4 d_5 d_6 d_7
//! </pre>
//!
//! along with the equations
//!
//! <pre>
//! (a+b)_8^2 = (a+b)_8  [booleanity check]
//! a' = sum_{i = 0}^7 (2^4)^i (a+b)_i;
//! a + b = 2^32 (a+b)_8 + a';
//! d = sum_{i = 0}^7 (2^4)^i d_i
//! d' = sum_{i = 0}^7 (2^4)^{(i + 4) mod 8} (a+b)_i
//! </pre>
//!
//! The (i + 4) mod 8 rotates the nybbles left by 4, which means bit-rotating by 4*4 = 16
//! as desired.
//!
//! The final line is a bit more complicated as we have to rotate by 7, which is not a multiple of 4.
//! We accomplish this as follows.
//!
//! Let's say we want to rotate the nybbles A_0, ..., A_7 left by 7.
//! First we'll rotate left by 4 to get
//!
//! A_7, A_0, A_1, ..., A_6
//!
//! Rename these as
//!
//! B_0, ..., B_7.
//!
//! We now want to left-rotate each B_i by 3.
//!
//! Let b_i be the low bit of B_i.
//! Then, the low 3 bits of B_i are
//! (B_i - b_i) / 2.
//!
//! The result will thus be
//!
//! 2^3 b_0 + (B_7 - b_7)/2,
//! 2^3 b_1 + (B_0 - b_0)/2,
//! 2^3 b_2 + (B_1 - b_1)/2,
//!     ...
//! 2^3 b_7 + (B_6 - b_6)/2,
//!
//! or re-writing in terms of our original nybbles A_i,
//!
//! 2^3 a_7 + (A_6 - a_6)/2,
//! 2^3 a_0 + (A_7 - a_7)/2,
//! 2^3 a_1 + (A_0 - a_0)/2,
//! 2^3 a_2 + (A_1 - a_1)/2,
//! 2^3 a_3 + (A_2 - a_2)/2,
//! 2^3 a_4 + (A_3 - a_3)/2,
//! 2^3 a_5 + (A_4 - a_4)/2,
//! 2^3 a_6 + (A_5 - a_5)/2,
//!
//! For neatness, letting (x, y, z) = (c', b', d''), the first 2 rows for the final
//! line will be,
//!
//! <pre>
//! x  y  z       (y^x')_0 (y^x')_1 (y^x')_2 (y^x')_3 (x+z)_0 (x+z)_1 (x+z)_2 (x+z)_3 y_0 y_1 y_2 y_3
//! x' _  (x+z)_8 (y^x')_4 (y^x')_5 (y^x')_6 (y^x')_7 (x+z)_4 (x+z)_5 (x+z)_6 (x+z)_7 y_4 y_5 y_6 y_7
//! </pre>
//!
//! but then we also need to perform the bit-rotate by 1.
//!
//! For this we'll add an additional 2 rows. It's probably possible to do it with just 1,
//! but I think we'd have to change our plookup setup somehow, or maybe expand the number of columns,
//! or allow access to the previous row.
//!
//! Let lo(n) be the low bit of the nybble n. The 2 rows will be
//!
//! y' (y^x')_0 (y^x')_1 (y^x')_2 (y^x')_3 lo((y^x')_0) lo((y^x')_1) lo((y^x')_2) lo((y^x')_3)
//! _  (y^x')_4 (y^x')_5 (y^x')_6 (y^x')_7 lo((y^x')_4) lo((y^x')_5) lo((y^x')_6) lo((y^x')_7)
//!
//! On each of them we'll do the plookups
//!
//! ((cols[1] - cols[5])/2, (cols[1] - cols[5])/2, 0) in XOR
//! ((cols[2] - cols[6])/2, (cols[2] - cols[6])/2, 0) in XOR
//! ((cols[3] - cols[7])/2, (cols[3] - cols[7])/2, 0) in XOR
//! ((cols[4] - cols[8])/2, (cols[4] - cols[8])/2, 0) in XOR
//!
//! which checks that ((y^x')_i - lo((y^x')_i)) is a nybble,
//! which guarantees that the low bit is computed correctly.
//!
//! There is no need to check nybbleness of (y^x')_i because those will be constrained to
//! be equal to the copies of those values from previous rows, which have already been
//! constrained for nybbleness (by the lookup in the XOR table).
//!
//! And we'll check that y' is the sum of the shifted nybbles.
//!

use std::marker::PhantomData;

use crate::expr::{Column, ConstantExpr as C, E};
use crate::gate::{CurrOrNext, GateType};
use ark_ff::{FftField, Field, Zero};
use CurrOrNext::*;

/// The lookup table for 4-bit xor.
/// Note that it is constructed so that (0, 0, 0) is the last position in the table.
///
/// This is because tables are extended to the full size of a column (essentially)
/// by padding them with their final value. And, having the value (0, 0, 0) here means
/// that when we commit to this table and use the dummy value in the `lookup_sorted`
/// columns, those entries that have the dummy value of
///
/// 0 = 0 + joint_combiner * 0 + joint_combiner^2 * 0
///
/// will translate into a scalar multiplication by 0, which is free.
pub fn xor_table<F: Field>() -> Vec<Vec<F>> {
    let mut res = vec![vec![]; 3];

    // XOR for all possible four-bit arguments.
    // I suppose this could be computed a bit faster using symmetry but it's quite
    // small (16*16 = 256 entries) so let's just keep it simple.
    for i in 0u32..0b10000 {
        for j in 0u32..0b10000 {
            res[0].push(F::from(i));
            res[1].push(F::from(j));
            res[2].push(F::from(i ^ j));
        }
    }

    for r in res.iter_mut().take(3) {
        r.reverse();
        // Just to be safe.
        assert!(r[r.len() - 1].is_zero());
    }
    res
}

/// This is just for tests. It doesn't set up the permutations
// TODO: add #[cfg(test)]
pub fn chacha20_gates() -> Vec<GateType> {
    let mut gs = vec![];
    for _ in 0..20 {
        use GateType::*;
        for _ in 0..4 {
            for &g in &[ChaCha0, ChaCha1, ChaCha2, ChaCha0, ChaChaFinal] {
                gs.push(g);
                gs.push(Zero);
            }
        }
    }
    gs
}

const CHACHA20_ROTATIONS: [u32; 4] = [16, 12, 8, 7];
const CHACHA20_QRS: [[usize; 4]; 8] = [
    [0, 4, 8, 12],
    [1, 5, 9, 13],
    [2, 6, 10, 14],
    [3, 7, 11, 15],
    [0, 5, 10, 15],
    [1, 6, 11, 12],
    [2, 7, 8, 13],
    [3, 4, 9, 14],
];

pub fn chacha20_rows<F: FftField>(s0: Vec<u32>) -> Vec<Vec<F>> {
    let mut rows = vec![];

    let mut s = s0;
    let mut line = |x: usize, y: usize, z: usize, k: u32| {
        let f = |t: u32| F::from(t);
        let nyb = |t: u32, i: usize| f((t >> (4 * i)) & 0b1111);

        let top_bit = (((s[x] as u64) + (s[z] as u64)) >> 32) as u32;
        let xprime = u32::wrapping_add(s[x], s[z]);
        let y_xor_xprime = s[y] ^ xprime;
        let yprime = y_xor_xprime.rotate_left(k);

        let yprime_in_row =
            // When k = 7, we use a ChaCha0 gate and throw away the yprime value
            // (which will need to be y_xor_xprime.rotate_left(16))
            // in the second row corresponding to that gate
            if k == 7 { y_xor_xprime.rotate_left(16) } else { yprime };

        rows.push(vec![
            f(s[x]),
            f(s[y]),
            f(s[z]),
            nyb(y_xor_xprime, 0),
            nyb(y_xor_xprime, 1),
            nyb(y_xor_xprime, 2),
            nyb(y_xor_xprime, 3),
            nyb(xprime, 0),
            nyb(xprime, 1),
            nyb(xprime, 2),
            nyb(xprime, 3),
            nyb(s[y], 0),
            nyb(s[y], 1),
            nyb(s[y], 2),
            nyb(s[y], 3),
        ]);
        rows.push(vec![
            f(xprime),
            f(yprime_in_row),
            f(top_bit),
            nyb(y_xor_xprime, 4),
            nyb(y_xor_xprime, 5),
            nyb(y_xor_xprime, 6),
            nyb(y_xor_xprime, 7),
            nyb(xprime, 4),
            nyb(xprime, 5),
            nyb(xprime, 6),
            nyb(xprime, 7),
            nyb(s[y], 4),
            nyb(s[y], 5),
            nyb(s[y], 6),
            nyb(s[y], 7),
        ]);

        s[x] = xprime;
        s[y] = yprime;

        if k == 7 {
            let lo = |t: u32, i: usize| f((t >> (4 * i)) & 1);
            rows.push(vec![
                f(yprime),
                nyb(y_xor_xprime, 0),
                nyb(y_xor_xprime, 1),
                nyb(y_xor_xprime, 2),
                nyb(y_xor_xprime, 3),
                lo(y_xor_xprime, 0),
                lo(y_xor_xprime, 1),
                lo(y_xor_xprime, 2),
                lo(y_xor_xprime, 3),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
            ]);
            rows.push(vec![
                F::zero(),
                nyb(y_xor_xprime, 4),
                nyb(y_xor_xprime, 5),
                nyb(y_xor_xprime, 6),
                nyb(y_xor_xprime, 7),
                lo(y_xor_xprime, 4),
                lo(y_xor_xprime, 5),
                lo(y_xor_xprime, 6),
                lo(y_xor_xprime, 7),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
            ]);
        }
    };

    let mut qr = |a, b, c, d| {
        line(a, d, b, CHACHA20_ROTATIONS[0]);
        line(c, b, d, CHACHA20_ROTATIONS[1]);
        line(a, d, b, CHACHA20_ROTATIONS[2]);
        line(c, b, d, CHACHA20_ROTATIONS[3]);
    };
    for _ in 0..10 {
        for [a, b, c, d] in CHACHA20_QRS {
            qr(a, b, c, d);
        }
    }

    rows
}

pub fn chacha20(mut s: Vec<u32>) -> Vec<u32> {
    let mut line = |x, y, z, k| {
        s[x] = u32::wrapping_add(s[x], s[z]);
        s[y] ^= s[x];
        let yy: u32 = s[y];
        s[y] = yy.rotate_left(k);
    };
    let mut qr = |a, b, c, d| {
        line(a, d, b, CHACHA20_ROTATIONS[0]);
        line(c, b, d, CHACHA20_ROTATIONS[1]);
        line(a, d, b, CHACHA20_ROTATIONS[2]);
        line(c, b, d, CHACHA20_ROTATIONS[3]);
    };
    for _ in 0..10 {
        for [a, b, c, d] in CHACHA20_QRS {
            qr(a, b, c, d);
        }
    }
    s
}

struct Helper<F>(PhantomData<F>);

impl<F> Helper<F>
where
    F: FftField,
{
    fn w(i: usize, r: CurrOrNext) -> E<F> {
        E::<F>::cell(Column::Witness(i), r)
    }

    /// 8-nybble sequences that are laid out as 4 nybbles per row over the two row,
    /// like y^x' or x+z
    fn chunks_over_2_rows(col_offset: usize) -> Vec<E<F>> {
        (0..8)
            .map(|i| {
                let r = if i < 4 { Curr } else { Next };
                Helper::w(col_offset + (i % 4), r)
            })
            .collect()
    }

    fn boolean(b: &E<F>) -> E<F> {
        b.clone() * b.clone() - b.clone()
    }

    fn combine_nybbles(ns: Vec<E<F>>) -> E<F> {
        ns.into_iter()
            .enumerate()
            .fold(E::zero(), |acc: E<F>, (i, t)| {
                acc + E::from(1 << (4 * i)) * t
            })
    }

    /// Constraints for the line L(x, x', y, y', z, k), where k = 4 * nybble_rotation
    fn line(alphas: &mut impl Iterator<Item = usize>, nybble_rotation: usize) -> E<F> {
        let y_xor_xprime_nybbles = Helper::chunks_over_2_rows(3);
        let x_plus_z_nybbles = Helper::chunks_over_2_rows(7);
        let y_nybbles = Helper::chunks_over_2_rows(11);

        let x_plus_z_overflow_bit = Helper::w(2, Next);

        let x = Helper::w(0, Curr);
        let xprime = Helper::w(0, Next);
        let y = Helper::w(1, Curr);
        let yprime = Helper::w(1, Next);
        let z = Helper::w(2, Curr);

        // Because the nybbles are little-endian, rotating the vector "right"
        // is equivalent to left-shifting the nybbles.
        let mut y_xor_xprime_rotated = y_xor_xprime_nybbles;
        y_xor_xprime_rotated.rotate_right(nybble_rotation);

        E::combine_constraints(
            alphas,
            vec![
                // booleanity of overflow bit
                Helper::boolean(&x_plus_z_overflow_bit),
                // x' = x + z (mod 2^32)
                Helper::combine_nybbles(x_plus_z_nybbles) - xprime.clone(),
                // Correctness of x+z nybbles
                xprime + E::from(1 << 32) * x_plus_z_overflow_bit - (x + z),
                // Correctness of y nybbles
                Helper::combine_nybbles(y_nybbles) - y,
                // y' = (y ^ x') <<< 4 * nybble_rotation
                Helper::combine_nybbles(y_xor_xprime_rotated) - yprime,
            ],
        )
    }
}

pub fn constraint<F: FftField>(alphas: &mut impl Iterator<Item = usize>) -> E<F> {
    let chacha_final = {
        let y_xor_xprime_nybbles = Helper::chunks_over_2_rows(1);
        let low_bits = Helper::chunks_over_2_rows(5);
        let yprime = Helper::w(0, Curr);

        let one_half = F::from(2u64).inverse().unwrap();

        // (y xor xprime) <<< 7
        // per the comment at the top of the file
        let y_xor_xprime_rotated: Vec<_> = [7, 0, 1, 2, 3, 4, 5, 6]
            .iter()
            .zip([6, 7, 0, 1, 2, 3, 4, 5].iter())
            .map(|(&i, &j)| -> E<F> {
                E::from(8) * low_bits[i].clone()
                    + E::Constant(C::Literal(one_half))
                        * (y_xor_xprime_nybbles[j].clone() - low_bits[j].clone())
            })
            .collect();

        let mut constraints: Vec<E<F>> = low_bits.iter().map(Helper::boolean).collect();
        constraints.push(Helper::combine_nybbles(y_xor_xprime_rotated) - yprime);
        E::combine_constraints(&mut alphas.by_ref().take(9), constraints)
    };

    let index = |g: GateType| E::cell(Column::Index(g), Curr);
    use GateType::*;
    vec![
        // a += b; d ^= a; d <<<= 16 (=4*4);
        index(ChaCha0) * Helper::line(&mut alphas.by_ref().take(5), 4),
        // c += d; b ^= c; b <<<= 12 (=3*4);
        index(ChaCha1) * Helper::line(&mut alphas.by_ref().take(5), 3),
        // a += b; d ^= a; d <<<= 8  (=2*4);
        index(ChaCha2) * Helper::line(&mut alphas.by_ref().take(5), 2),
        // The last line, namely,
        // c += d; b ^= c; b <<<= 7;
        // is special.
        // We don't use the y' value computed by this one, so we
        // will use a ChaCha0 gate to compute the nybbles of
        // all the relevant values, and the xors, and then do
        // the shifting using a ChaChaFinal gate.
        index(ChaChaFinal) * chacha_final,
    ]
    .into_iter()
    .fold(0.into(), |acc, x| acc + x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polynomials::chacha::constraint;
    use crate::{
        expr::{Column, Constants, PolishToken},
        gate::LookupInfo,
        nolookup::scalars::{LookupEvaluations, ProofEvaluations},
        wires::*,
    };
    use ark_ff::UniformRand;
    use ark_poly::{EvaluationDomain, Radix2EvaluationDomain as D};
    use array_init::array_init;
    use mina_curves::pasta::fp::Fp as F;
    use rand::{rngs::StdRng, SeedableRng};
    use std::fmt::{Display, Formatter};

    struct Polish(Vec<PolishToken<F>>);
    impl Display for Polish {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[")?;
            for x in self.0.iter() {
                match x {
                    PolishToken::Literal(a) => write!(f, "{}, ", a)?,
                    PolishToken::Add => write!(f, "+, ")?,
                    PolishToken::Mul => write!(f, "*, ")?,
                    PolishToken::Sub => write!(f, "-, ")?,
                    x => write!(f, "{:?}, ", x)?,
                }
            }
            write!(f, "]")?;
            Ok(())
        }
    }

    #[test]
    fn chacha_linearization() {
        let lookup_info = LookupInfo::<F>::create();

        let evaluated_cols = {
            let mut h = std::collections::HashSet::new();
            // use Column::*;
            for i in 0..COLUMNS {
                h.insert(Column::Witness(i));
            }
            for i in 0..(lookup_info.max_per_row + 1) {
                h.insert(Column::LookupSorted(i));
            }
            h.insert(Column::Z);
            h.insert(Column::LookupAggreg);
            h.insert(Column::LookupTable);
            h.insert(Column::Index(GateType::Poseidon));
            h.insert(Column::Index(GateType::Generic));
            h
        };
        let mut powers = 0..24;
        let expr = constraint(&mut powers);
        let linearized = expr.linearize(evaluated_cols).unwrap();
        let _expr_polish = expr.to_polish();
        let linearized_polish = linearized.map(|e| e.to_polish());

        let rng = &mut StdRng::from_seed([0u8; 32]);

        let d = D::new(1024).unwrap();

        let pt = F::rand(rng);
        let mut eval = || ProofEvaluations {
            w: array_init(|_| F::rand(rng)),
            z: F::rand(rng),
            s: array_init(|_| F::rand(rng)),
            generic_selector: F::zero(),
            poseidon_selector: F::zero(),
            lookup: Some(LookupEvaluations {
                sorted: (0..(lookup_info.max_per_row + 1))
                    .map(|_| F::rand(rng))
                    .collect(),
                aggreg: F::rand(rng),
                table: F::rand(rng),
            }),
        };
        let evals = vec![eval(), eval()];

        let constants = Constants {
            alpha: F::rand(rng),
            beta: F::rand(rng),
            gamma: F::rand(rng),
            joint_combiner: F::rand(rng),
            endo_coefficient: F::zero(),
            mds: vec![],
        };

        assert_eq!(
            linearized
                .constant_term
                .evaluate_(d, pt, &evals, &constants)
                .unwrap(),
            PolishToken::evaluate(&linearized_polish.constant_term, d, pt, &evals, &constants)
                .unwrap()
        );

        linearized
            .index_terms
            .iter()
            .zip(linearized_polish.index_terms.iter())
            .for_each(|((c1, e1), (c2, e2))| {
                assert_eq!(c1, c2);
                println!("{:?} ?", c1);
                let x1 = e1.evaluate_(d, pt, &evals, &constants).unwrap();
                let x2 = PolishToken::evaluate(e2, d, pt, &evals, &constants).unwrap();
                if x1 != x2 {
                    println!("e1: {}", e1);
                    println!("e2: {}", Polish(e2.clone()));
                    println!("Polish evaluation differed for {:?}: {} != {}", c1, x1, x2);
                } else {
                    println!("{:?} OK", c1);
                }
            });

        /*
        assert_eq!(
            expr.evaluate_(d, pt, &evals, &constants).unwrap(),
            PolishToken::evaluate(&expr_polish, d, pt, &evals, &constants).unwrap());
            */
    }
}
