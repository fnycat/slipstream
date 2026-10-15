#![no_main]

use std::sync::Arc;

use libfuzzer_sys::{fuzz_mutator, fuzz_target};
use slipstream_ir::node::arena::IrArena;
use slipstream_shared::RefCursor;

fuzz_target!(|data: &[u8]| {
    let reader = RefCursor::new(Arc::from(data));
    let arena = IrArena::new();

    let _ = slipstream_ir::node::root::deserialize_maybe_compressed(reader, &arena, String::from("fuzzed.szs"));
});