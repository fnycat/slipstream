#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mktools::error::EditorResult;
use mktools::run;

fn main() -> EditorResult<()> {
    run()
}
