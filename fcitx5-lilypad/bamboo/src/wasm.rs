//! WebAssembly (WASM) bindings for Bamboo Core.
//!
//! This module provides a high-level wrapper around the [`crate::Engine`] for use in Web browsers
//! and other WASM environments via `wasm-bindgen`.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// A WebAssembly-compatible wrapper for the Bamboo engine.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct BambooWasmEngine {
    inner: crate::engine::Engine,
}

#[cfg(feature = "wasm")]
impl Default for BambooWasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl BambooWasmEngine {
    /// Creates a new engine instance with the default Telex input method.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: crate::engine::Engine::new(crate::input_method::InputMethod::telex()) }
    }

    /// Processes a single character and returns the current transformed word.
    pub fn process_key(&mut self, key: char) -> String {
        self.inner.process_key(key, crate::mode::Mode::Vietnamese);
        self.inner.output().into_owned()
    }

    /// Resets the engine state, clearing all committed and active text.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Returns the current transformed word output.
    pub fn output(&self) -> String {
        self.inner.output().into_owned()
    }
}
