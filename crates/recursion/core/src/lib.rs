use p3_field::PrimeField64;
use serde::{Deserialize, Serialize};
use sp1_derive::AlignedBorrow;

use crate::air::{Block, RecursionPublicValues};

pub mod air;
pub mod builder;
pub mod chips;
pub mod machine;
pub mod runtime;
pub mod shape;
pub mod stark;
pub mod sys;

pub use runtime::*;

// Re-export the stark stuff from `sp1_recursion_core` for now, until we will migrate it here.
// pub use sp1_recursion_core::stark;

use crate::chips::poseidon2_skinny::WIDTH;

#[derive(
    AlignedBorrow, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
#[repr(transparent)]
pub struct Address<F>(pub F);

impl<F: PrimeField64> Address<F> {
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0.as_canonical_u64() as usize
    }
}

// -------------------------------------------------------------------------------------------------

/// The inputs and outputs to an operation of the base field ALU.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct BaseAluIo<V> {
    pub out: V,
    pub in1: V,
    pub in2: V,
}

pub type BaseAluEvent<F> = BaseAluIo<F>;

/// An instruction invoking the extension field ALU.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct BaseAluInstr<F> {
    pub opcode: BaseAluOpcode,
    pub mult: F,
    pub addrs: BaseAluIo<Address<F>>,
}

// -------------------------------------------------------------------------------------------------

/// The inputs and outputs to an operation of the extension field ALU.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct ExtAluIo<V> {
    pub out: V,
    pub in1: V,
    pub in2: V,
}

pub type ExtAluEvent<F> = ExtAluIo<Block<F>>;

/// An instruction invoking the extension field ALU.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ExtAluInstr<F> {
    pub opcode: ExtAluOpcode,
    pub mult: F,
    pub addrs: ExtAluIo<Address<F>>,
}

// -------------------------------------------------------------------------------------------------

/// The inputs and outputs to the manual memory management/memory initialization table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemIo<V> {
    pub inner: V,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemInstr<F> {
    pub addrs: MemIo<Address<F>>,
    pub vals: MemIo<Block<F>>,
    pub mult: F,
    pub kind: MemAccessKind,
}

pub type MemEvent<F> = MemIo<Block<F>>;

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemAccessKind {
    Read,
    Write,
}

/// The inputs and outputs to a Poseidon2 permutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Poseidon2Io<V> {
    pub input: [V; WIDTH],
    pub output: [V; WIDTH],
}

/// An instruction invoking the Poseidon2 permutation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Poseidon2SkinnyInstr<F> {
    pub addrs: Poseidon2Io<Address<F>>,
    pub mults: [F; WIDTH],
}

pub type Poseidon2Event<F> = Poseidon2Io<F>;

/// The inputs and outputs to a select operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct SelectIo<V> {
    pub bit: V,
    pub out1: V,
    pub out2: V,
    pub in1: V,
    pub in2: V,
}

/// An instruction invoking the select operation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SelectInstr<F> {
    pub addrs: SelectIo<Address<F>>,
    pub mult1: F,
    pub mult2: F,
}

/// The event encoding the inputs and outputs of a select operation.
pub type SelectEvent<F> = SelectIo<F>;

/// The inputs and outputs to an exp-reverse-bits operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpReverseBitsIo<V> {
    pub base: V,
    // The bits of the exponent in little-endian order in a vec.
    pub exp: Vec<V>,
    pub result: V,
}

pub type Poseidon2WideEvent<F> = Poseidon2Io<F>;
pub type Poseidon2Instr<F> = Poseidon2SkinnyInstr<F>;

/// An instruction invoking the exp-reverse-bits operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpReverseBitsInstr<F> {
    pub addrs: ExpReverseBitsIo<Address<F>>,
    pub mult: F,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ExpReverseBitsInstrC<'a, F> {
    pub base: &'a Address<F>,
    pub exp_ptr: *const Address<F>,
    pub exp_len: usize,
    pub result: &'a Address<F>,
    pub mult: &'a F,
}

impl<F> ExpReverseBitsInstr<F> {
    pub fn to_c(&self) -> ExpReverseBitsInstrC<'_, F> {
        ExpReverseBitsInstrC {
            base: &self.addrs.base,
            exp_ptr: self.addrs.exp.as_ptr(),
            exp_len: self.addrs.exp.len(),
            result: &self.addrs.result,
            mult: &self.mult,
        }
    }
}

/// The event encoding the inputs and outputs of an exp-reverse-bits operation. The `len` operand is
/// now stored as the length of the `exp` field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpReverseBitsEvent<F> {
    pub base: F,
    pub exp: Vec<F>,
    pub result: F,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ExpReverseBitsEventC<'a, F> {
    pub base: &'a F,
    pub exp_ptr: *const F,
    pub exp_len: usize,
    pub result: &'a F,
}

impl<F> ExpReverseBitsEvent<F> {
    pub fn to_c(&self) -> ExpReverseBitsEventC<'_, F> {
        ExpReverseBitsEventC {
            base: &self.base,
            exp_ptr: self.exp.as_ptr(),
            exp_len: self.exp.len(),
            result: &self.result,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FriFoldIo<V> {
    pub ext_single: FriFoldExtSingleIo<Block<V>>,
    pub ext_vec: FriFoldExtVecIo<Vec<Block<V>>>,
    pub base_single: FriFoldBaseIo<V>,
}

/// The extension-field-valued single inputs to the FRI fold operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct FriFoldExtSingleIo<V> {
    pub z: V,
    pub alpha: V,
}

/// The extension-field-valued vector inputs to the FRI fold operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct FriFoldExtVecIo<V> {
    pub mat_opening: V,
    pub ps_at_z: V,
    pub alpha_pow_input: V,
    pub ro_input: V,
    pub alpha_pow_output: V,
    pub ro_output: V,
}

/// The base-field-valued inputs to the FRI fold operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct FriFoldBaseIo<V> {
    pub x: V,
}

/// An instruction invoking the FRI fold operation. Addresses for extension field elements are of
/// the same type as for base field elements.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FriFoldInstr<F> {
    pub base_single_addrs: FriFoldBaseIo<Address<F>>,
    pub ext_single_addrs: FriFoldExtSingleIo<Address<F>>,
    pub ext_vec_addrs: FriFoldExtVecIo<Vec<Address<F>>>,
    pub alpha_pow_mults: Vec<F>,
    pub ro_mults: Vec<F>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct FriFoldInstrC<'a, F> {
    pub base_single_addrs: &'a FriFoldBaseIo<Address<F>>,
    pub ext_single_addrs: &'a FriFoldExtSingleIo<Address<F>>,

    pub ext_vec_addrs_mat_opening_ptr: *const Address<F>,
    pub ext_vec_addrs_mat_opening_len: usize,
    pub ext_vec_addrs_ps_at_z_ptr: *const Address<F>,
    pub ext_vec_addrs_ps_at_z_len: usize,
    pub ext_vec_addrs_alpha_pow_input_ptr: *const Address<F>,
    pub ext_vec_addrs_alpha_pow_input_len: usize,
    pub ext_vec_addrs_ro_input_ptr: *const Address<F>,
    pub ext_vec_addrs_ro_input_len: usize,
    pub ext_vec_addrs_alpha_pow_output_ptr: *const Address<F>,
    pub ext_vec_addrs_alpha_pow_output_len: usize,
    pub ext_vec_addrs_ro_output_ptr: *const Address<F>,
    pub ext_vec_addrs_ro_output_len: usize,

    pub alpha_pow_mults_ptr: *const F,
    pub alpha_pow_mults_len: usize,

    pub ro_mults_ptr: *const F,
    pub ro_mults_len: usize,
}

impl<F> FriFoldInstr<F> {
    pub fn to_c(&self) -> FriFoldInstrC<'_, F> {
        FriFoldInstrC {
            base_single_addrs: &self.base_single_addrs,
            ext_single_addrs: &self.ext_single_addrs,

            ext_vec_addrs_mat_opening_ptr: self.ext_vec_addrs.mat_opening.as_ptr(),
            ext_vec_addrs_mat_opening_len: self.ext_vec_addrs.mat_opening.len(),
            ext_vec_addrs_ps_at_z_ptr: self.ext_vec_addrs.ps_at_z.as_ptr(),
            ext_vec_addrs_ps_at_z_len: self.ext_vec_addrs.ps_at_z.len(),
            ext_vec_addrs_alpha_pow_input_ptr: self.ext_vec_addrs.alpha_pow_input.as_ptr(),
            ext_vec_addrs_alpha_pow_input_len: self.ext_vec_addrs.alpha_pow_input.len(),
            ext_vec_addrs_ro_input_ptr: self.ext_vec_addrs.ro_input.as_ptr(),
            ext_vec_addrs_ro_input_len: self.ext_vec_addrs.ro_input.len(),
            ext_vec_addrs_alpha_pow_output_ptr: self.ext_vec_addrs.alpha_pow_output.as_ptr(),
            ext_vec_addrs_alpha_pow_output_len: self.ext_vec_addrs.alpha_pow_output.len(),
            ext_vec_addrs_ro_output_ptr: self.ext_vec_addrs.ro_output.as_ptr(),
            ext_vec_addrs_ro_output_len: self.ext_vec_addrs.ro_output.len(),

            alpha_pow_mults_ptr: self.alpha_pow_mults.as_ptr(),
            alpha_pow_mults_len: self.alpha_pow_mults.len(),

            ro_mults_ptr: self.ro_mults.as_ptr(),
            ro_mults_len: self.ro_mults.len(),
        }
    }
}

/// The event encoding the data of a single iteration within the FRI fold operation.
/// For any given event, we are accessing a single element of the `Vec` inputs, so that the event
/// is not a type alias for `FriFoldIo` like many of the other events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct FriFoldEvent<F> {
    pub base_single: FriFoldBaseIo<F>,
    pub ext_single: FriFoldExtSingleIo<Block<F>>,
    pub ext_vec: FriFoldExtVecIo<Block<F>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchFRIIo<V> {
    pub ext_single: BatchFRIExtSingleIo<Block<V>>,
    pub ext_vec: BatchFRIExtVecIo<Vec<Block<V>>>,
    pub base_vec: BatchFRIBaseVecIo<V>,
}

/// The extension-field-valued single inputs to the batch FRI operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct BatchFRIExtSingleIo<V> {
    pub acc: V,
}

/// The extension-field-valued vector inputs to the batch FRI operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct BatchFRIExtVecIo<V> {
    pub p_at_z: V,
    pub alpha_pow: V,
}

/// The base-field-valued vector inputs to the batch FRI operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct BatchFRIBaseVecIo<V> {
    pub p_at_x: V,
}

/// An instruction invoking the batch FRI operation. Addresses for extension field elements are of
/// the same type as for base field elements.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct BatchFRIInstr<F> {
    pub base_vec_addrs: BatchFRIBaseVecIo<Vec<Address<F>>>,
    pub ext_single_addrs: BatchFRIExtSingleIo<Address<F>>,
    pub ext_vec_addrs: BatchFRIExtVecIo<Vec<Address<F>>>,
    pub acc_mult: F,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct BatchFRIInstrC<'a, F> {
    pub base_vec_addrs_p_at_x_ptr: *const Address<F>,
    pub base_vec_addrs_p_at_x_len: usize,

    pub ext_single_addrs: &'a BatchFRIExtSingleIo<Address<F>>,

    pub ext_vec_addrs_p_at_z_ptr: *const Address<F>,
    pub ext_vec_addrs_p_at_z_len: usize,
    pub ext_vec_addrs_alpha_pow_ptr: *const Address<F>,
    pub ext_vec_addrs_alpha_pow_len: usize,

    pub acc_mult: &'a F,
}

impl<F> BatchFRIInstr<F> {
    pub fn to_c(&self) -> BatchFRIInstrC<'_, F> {
        BatchFRIInstrC {
            base_vec_addrs_p_at_x_ptr: self.base_vec_addrs.p_at_x.as_ptr(),
            base_vec_addrs_p_at_x_len: self.base_vec_addrs.p_at_x.len(),

            ext_single_addrs: &self.ext_single_addrs,

            ext_vec_addrs_p_at_z_ptr: self.ext_vec_addrs.p_at_z.as_ptr(),
            ext_vec_addrs_p_at_z_len: self.ext_vec_addrs.p_at_z.len(),
            ext_vec_addrs_alpha_pow_ptr: self.ext_vec_addrs.alpha_pow.as_ptr(),
            ext_vec_addrs_alpha_pow_len: self.ext_vec_addrs.alpha_pow.len(),

            acc_mult: &self.acc_mult,
        }
    }
}

/// The event encoding the data of a single iteration within the batch FRI operation.
/// For any given event, we are accessing a single element of the `Vec` inputs, so that the event
/// is not a type alias for `BatchFRIIo` like many of the other events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct BatchFRIEvent<F> {
    pub base_vec: BatchFRIBaseVecIo<F>,
    pub ext_single: BatchFRIExtSingleIo<Block<F>>,
    pub ext_vec: BatchFRIExtVecIo<Block<F>>,
}

/// An instruction that will save the public values to the execution record and will commit to
/// it's digest.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitPublicValuesInstr<F> {
    pub pv_addrs: RecursionPublicValues<Address<F>>,
}

/// The event for committing to the public values.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CommitPublicValuesEvent<F> {
    pub public_values: RecursionPublicValues<F>,
}
