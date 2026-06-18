mod decoy;
mod decoy_sections;
mod ident;
mod numbers;
mod profile;
mod rename;

pub use decoy::{decoy_block, DecoyReport};
pub use ident::IdentGenerator;
pub use numbers::{rewrite_number_literals, NumberEncoder};
pub use profile::{
    BytecodeLayout, ConstantLayout, HardeningProfile, ProfileStats, RuntimeTemplateVariant,
};
pub use rename::{rename_identifiers, Binding};
