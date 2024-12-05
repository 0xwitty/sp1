pub mod alu_base;
pub mod alu_ext;
pub mod batch_fri;
pub mod exp_reverse_bits;
pub mod fri_fold;
pub mod mem;
pub mod poseidon2_skinny;
pub mod poseidon2_wide;
pub mod public_values;
pub mod select;

pub mod test_fixtures {
    use crate::*;
    use p3_baby_bear::BabyBear;
    use p3_field::{AbstractField, Field, PrimeField32};
    use rand::{prelude::SliceRandom, rngs::StdRng, Rng, SeedableRng};

    const SEED: u64 = 12345;
    const MIN_TEST_CASES: usize = 1000;
    const MAX_TEST_CASES: usize = 10000;

    pub fn shard() -> ExecutionRecord<BabyBear> {
        ExecutionRecord {
            base_alu_events: base_alu_events(),
            ext_alu_events: ext_alu_events(),
            batch_fri_events: batch_fri_events(),
            exp_reverse_bits_len_events: exp_reverse_bits_events(),
            fri_fold_events: fri_fold_events(),
            ..Default::default()
        }
    }

    pub fn program() -> RecursionProgram<BabyBear> {
        let mut instructions = Vec::new();
        instructions.push(base_alu_instructions());
        instructions.push(ext_alu_instructions());
        instructions.push(batch_fri_instructions());
        instructions.push(exp_reverse_bits_instructions());
        instructions.push(fri_fold_instructions());

        let mut rng = StdRng::seed_from_u64(SEED);
        let mut flattened: Vec<_> = instructions.into_iter().flatten().collect();
        flattened.shuffle(&mut rng);

        RecursionProgram { instructions: flattened, ..Default::default() }
    }

    pub fn default_execution_record() -> ExecutionRecord<BabyBear> {
        ExecutionRecord::<BabyBear>::default()
    }

    fn initialize() -> (StdRng, usize) {
        let mut rng = StdRng::seed_from_u64(SEED);
        let num_test_cases = rng.gen_range(MIN_TEST_CASES..=MAX_TEST_CASES);
        (rng, num_test_cases)
    }

    fn base_alu_events() -> Vec<BaseAluIo<BabyBear>> {
        let (mut rng, num_test_cases) = initialize();
        let mut events = Vec::with_capacity(num_test_cases);
        for _ in 0..num_test_cases {
            let in1 = BabyBear::from_wrapped_u32(rng.gen());
            let in2 = BabyBear::from_wrapped_u32(rng.gen());
            let out = match rng.gen_range(0..4) {
                0 => in1 + in2, // Add
                1 => in1 - in2, // Sub
                2 => in1 * in2, // Mul
                _ => {
                    let in2 = if in2.is_zero() { BabyBear::one() } else { in2 };
                    in1 / in2
                }
            };
            events.push(BaseAluIo { out, in1, in2 });
        }
        events
    }

    fn ext_alu_events() -> Vec<ExtAluIo<Block<BabyBear>>> {
        let (_, num_test_cases) = initialize();
        let mut events = Vec::with_capacity(num_test_cases);
        for _ in 0..num_test_cases {
            events.push(ExtAluIo {
                out: BabyBear::one().into(),
                in1: BabyBear::one().into(),
                in2: BabyBear::one().into(),
            });
        }
        events
    }

    fn batch_fri_events() -> Vec<BatchFRIEvent<BabyBear>> {
        let (_, num_test_cases) = initialize();
        let mut events = Vec::with_capacity(num_test_cases);
        for _ in 0..num_test_cases {
            events.push(BatchFRIEvent {
                ext_single: BatchFRIExtSingleIo { acc: Block::default() },
                ext_vec: BatchFRIExtVecIo { alpha_pow: Block::default(), p_at_z: Block::default() },
                base_vec: BatchFRIBaseVecIo { p_at_x: BabyBear::one() },
            });
        }
        events
    }

    fn exp_reverse_bits_events() -> Vec<ExpReverseBitsEvent<BabyBear>> {
        let (mut rng, num_test_cases) = initialize();
        let mut events = Vec::with_capacity(num_test_cases);
        for _ in 0..num_test_cases {
            let base = BabyBear::from_wrapped_u32(rng.gen());
            let len = rng.gen_range(1..8); // Random length between 1 and 7 bits
            let exp: Vec<BabyBear> =
                (0..len).map(|_| BabyBear::from_canonical_u32(rng.gen_range(0..2))).collect();
            let exp_num = exp
                .iter()
                .enumerate()
                .fold(0u32, |acc, (i, &bit)| acc + (bit.as_canonical_u32() << i));
            let result = base.exp_u64(exp_num as u64);

            events.push(ExpReverseBitsEvent { base, exp, result });
        }
        events
    }

    fn fri_fold_events() -> Vec<FriFoldEvent<BabyBear>> {
        let (mut rng, num_test_cases) = initialize();
        let mut events = Vec::with_capacity(num_test_cases);
        let random_block =
            |rng: &mut StdRng| Block::from([BabyBear::from_wrapped_u32(rng.gen()); 4]);
        for _ in 0..num_test_cases {
            events.push(FriFoldEvent {
                base_single: FriFoldBaseIo { x: BabyBear::from_wrapped_u32(rng.gen()) },
                ext_single: FriFoldExtSingleIo {
                    z: random_block(&mut rng),
                    alpha: random_block(&mut rng),
                },
                ext_vec: FriFoldExtVecIo {
                    mat_opening: random_block(&mut rng),
                    ps_at_z: random_block(&mut rng),
                    alpha_pow_input: random_block(&mut rng),
                    ro_input: random_block(&mut rng),
                    alpha_pow_output: random_block(&mut rng),
                    ro_output: random_block(&mut rng),
                },
            });
        }
        events
    }

    fn base_alu_instructions() -> Vec<Instruction<BabyBear>> {
        let (mut rng, num_test_cases) = initialize();
        let mut instructions = Vec::with_capacity(num_test_cases);
        for _ in 0..num_test_cases {
            let opcode = match rng.gen_range(0..4) {
                0 => BaseAluOpcode::AddF,
                1 => BaseAluOpcode::SubF,
                2 => BaseAluOpcode::MulF,
                _ => BaseAluOpcode::DivF,
            };
            instructions.push(Instruction::BaseAlu(BaseAluInstr {
                opcode,
                mult: BabyBear::from_wrapped_u32(rng.gen()),
                addrs: BaseAluIo {
                    out: Address(BabyBear::from_wrapped_u32(rng.gen())),
                    in1: Address(BabyBear::from_wrapped_u32(rng.gen())),
                    in2: Address(BabyBear::from_wrapped_u32(rng.gen())),
                },
            }));
        }
        instructions
    }

    fn ext_alu_instructions() -> Vec<Instruction<BabyBear>> {
        let (mut rng, num_test_cases) = initialize();
        let mut instructions = Vec::with_capacity(num_test_cases);
        for _ in 0..num_test_cases {
            let opcode = match rng.gen_range(0..4) {
                0 => ExtAluOpcode::AddE,
                1 => ExtAluOpcode::SubE,
                2 => ExtAluOpcode::MulE,
                _ => ExtAluOpcode::DivE,
            };
            instructions.push(Instruction::ExtAlu(ExtAluInstr {
                opcode,
                mult: BabyBear::from_wrapped_u32(rng.gen()),
                addrs: ExtAluIo {
                    out: Address(BabyBear::from_wrapped_u32(rng.gen())),
                    in1: Address(BabyBear::from_wrapped_u32(rng.gen())),
                    in2: Address(BabyBear::from_wrapped_u32(rng.gen())),
                },
            }));
        }
        instructions
    }

    fn batch_fri_instructions() -> Vec<Instruction<BabyBear>> {
        let (mut rng, num_test_cases) = initialize();
        let mut instructions = Vec::with_capacity(num_test_cases);
        for _ in 0..num_test_cases {
            let len = rng.gen_range(1..5); // Random number of addresses in vectors
            let p_at_x = (0..len).map(|_| Address(BabyBear::from_wrapped_u32(rng.gen()))).collect();
            let alpha_pow =
                (0..len).map(|_| Address(BabyBear::from_wrapped_u32(rng.gen()))).collect();
            let p_at_z = (0..len).map(|_| Address(BabyBear::from_wrapped_u32(rng.gen()))).collect();
            let acc = Address(BabyBear::from_wrapped_u32(rng.gen()));
            instructions.push(Instruction::BatchFRI(Box::new(BatchFRIInstr {
                base_vec_addrs: BatchFRIBaseVecIo { p_at_x },
                ext_single_addrs: BatchFRIExtSingleIo { acc },
                ext_vec_addrs: BatchFRIExtVecIo { alpha_pow, p_at_z },
                acc_mult: BabyBear::one(), // BatchFRI always uses mult of 1
            })));
        }
        instructions
    }

    fn exp_reverse_bits_instructions() -> Vec<Instruction<BabyBear>> {
        let (mut rng, num_test_cases) = initialize();
        let mut instructions = Vec::with_capacity(num_test_cases);
        for _ in 0..num_test_cases {
            let len = rng.gen_range(1..8); // Random length between 1 and 7 bits
            let exp: Vec<Address<BabyBear>> =
                (0..len).map(|_| Address(BabyBear::from_wrapped_u32(rng.gen()))).collect();
            let base = Address(BabyBear::from_wrapped_u32(rng.gen()));
            let result = Address(BabyBear::from_wrapped_u32(rng.gen()));
            let mult = BabyBear::from_wrapped_u32(rng.gen());
            instructions.push(Instruction::ExpReverseBitsLen(ExpReverseBitsInstr {
                addrs: ExpReverseBitsIo { base, exp, result },
                mult,
            }));
        }
        instructions
    }

    fn fri_fold_instructions() -> Vec<Instruction<BabyBear>> {
        let (mut rng, num_test_cases) = initialize();
        let mut instructions = Vec::with_capacity(num_test_cases);
        let random_addr = |rng: &mut StdRng| Address(BabyBear::from_wrapped_u32(rng.gen()));
        let random_addrs =
            |rng: &mut StdRng, len: usize| (0..len).map(|_| random_addr(rng)).collect();
        for _ in 0..num_test_cases {
            let len = rng.gen_range(1..5); // Random vector length
            instructions.push(Instruction::FriFold(Box::new(FriFoldInstr {
                base_single_addrs: FriFoldBaseIo { x: random_addr(&mut rng) },
                ext_single_addrs: FriFoldExtSingleIo {
                    z: random_addr(&mut rng),
                    alpha: random_addr(&mut rng),
                },
                ext_vec_addrs: FriFoldExtVecIo {
                    mat_opening: random_addrs(&mut rng, len),
                    ps_at_z: random_addrs(&mut rng, len),
                    alpha_pow_input: random_addrs(&mut rng, len),
                    ro_input: random_addrs(&mut rng, len),
                    alpha_pow_output: random_addrs(&mut rng, len),
                    ro_output: random_addrs(&mut rng, len),
                },
                alpha_pow_mults: vec![BabyBear::one(); len],
                ro_mults: vec![BabyBear::one(); len],
            })));
        }
        instructions
    }
}
