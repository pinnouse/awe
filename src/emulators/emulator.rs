use std::collections::HashMap;
use wasm_bindgen::JsValue;
use crate::wasm_bindgen;

/// Emulator trait to have standard calling interface across implementations.
pub trait Emulator {
    /// Initialize a new emulator
    fn e_new() -> Self;

    /// Load arbitrary binary
    fn e_load(&mut self, data: Vec<u8>);

    /// Execute an opcode, largest accepted is a 64-bit command
    fn e_execute_op(&mut self, opcode: u64);

    /// Update loop
    fn e_update(&mut self);

    /// Arbitrarily set properties/metadata for the emulator.
    /// Could be screen sizes, refresh rate -- any arbitrary data;
    fn e_set_metadata(&mut self, metadata: HashMap<String, JsValue>);

    /// Draw loop
    fn e_draw(&mut self);

    /// Set input maybe key or gamepad
    fn e_set_input(&mut self);

    /// Arbitrary reset function to reset the state of the emulator
    fn e_reset(&mut self);
}