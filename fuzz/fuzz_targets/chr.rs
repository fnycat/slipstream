#![no_main]

use std::sync::Arc;

use libfuzzer_sys::fuzz_target;
use slipstream_ir::brres::chr;
use slipstream_ir::node::arena::IrArena;
use slipstream_shared::RefCursor;

fuzz_target!(|data: &[u8]| {
    let mut reader = RefCursor::new(Arc::from(data));
    let arena = IrArena::new();
    let _ = chr::deserialize(&mut reader, None, &arena, String::from("chr_fuzz"));
});
