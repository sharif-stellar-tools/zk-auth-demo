#![no_std]
use soroban_sdk::{contract, contracterror};
#[contracterror]
pub enum Error { InvalidProof = 1 }