use crate::{
    mips::{
        column::MIPS_COLUMNS,
        constraints::Env,
        interpreter::{interpret_instruction, Instruction},
    },
    tester::{Circuit, CircuitPad},
};
use ark_ff::Field;
use kimchi_msm::witness::Witness;
use std::{array, collections::HashMap};
use strum::IntoEnumIterator;

/// The Keccak circuit
pub type MIPSCircuit<F> = Circuit<MIPS_COLUMNS, Instruction, F>;

impl<F: Field> CircuitPad<MIPS_COLUMNS, Instruction, F, Env<F>> for MIPSCircuit<F> {
    fn new(domain_size: usize, env: &mut Env<F>) -> Self {
        let mut circuit = Self {
            domain_size,
            witness: HashMap::new(),
            constraints: Default::default(),
            lookups: Default::default(),
        };

        for instr in Instruction::iter().flat_map(|x| x.into_iter()) {
            circuit.witness.insert(
                instr,
                Witness {
                    cols: Box::new(std::array::from_fn(|_| Vec::with_capacity(domain_size))),
                },
            );
            interpret_instruction(env, instr);
            circuit.constraints.insert(instr, env.constraints.clone());
            circuit.lookups.insert(instr, env.lookups.clone());
            env.scratch_state_idx = 0; // Reset the scratch state index for the next instruction
            env.constraints = vec![]; // Clear the constraints for the next instruction
            env.lookups = vec![]; // Clear the lookups for the next instruction
        }
        circuit
    }

    fn push_row(&mut self, instr: Instruction, row: &[F; MIPS_COLUMNS]) {
        self.witness.entry(instr).and_modify(|wit| {
            for (i, value) in row.iter().enumerate() {
                if wit.cols[i].len() < wit.cols[i].capacity() {
                    wit.cols[i].push(*value);
                }
            }
        });
    }

    fn pad_with_row(&mut self, step: Instruction, row: &[F; MIPS_COLUMNS]) -> bool {
        let rows_left = self.domain_size - self.witness[&step].cols[0].len();
        if rows_left == 0 {
            return false;
        }
        for _ in 0..rows_left {
            self.push_row(step, row);
        }
        true
    }

    fn pad_with_zeros(&mut self, instr: Instruction) -> bool {
        let rows_left = self.domain_size - self.witness[&instr].cols[0].len();
        if rows_left == 0 {
            return false;
        }
        self.witness.entry(instr).and_modify(|wit| {
            for col in wit.cols.iter_mut() {
                col.extend((0..rows_left).map(|_| F::zero()));
            }
        });
        true
    }

    fn pad_dummy(&mut self, step: Instruction) -> bool {
        if self.witness_is_empty(step) {
            false
        } else {
            let row = array::from_fn(|i| self.witness[&step].cols[i][0]);
            self.pad_with_row(step, &row)
        }
    }

    fn pad_witnesses(&mut self) {
        for step in Instruction::iter().flat_map(|step| step.into_iter()) {
            self.pad_dummy(step);
        }
    }
}
