#![feature(error_generic_member_access)]

pub mod arc;
pub mod brres;
pub mod chr0;
pub mod encoding;
pub mod error;
pub mod mdl0;
pub mod pat0;
pub mod yaz0;

#[cfg(test)]
mod test {
    use crate::{arc, brres, encoding::Decode, yaz0::decompress};
    use std::io::Cursor;

    fn setup_tracing() {
        use tracing_subscriber::layer::SubscriberExt;

        color_eyre::config::HookBuilder::new()
            .panic_section("report this issue at https://github.com/RadiatedMonkey/mktools")
            // .issue_url("https://github.com/RadiatedMonkey/mktools/issues/new")
            // .add_issue_metadata("version", "v0.1.0")
            .display_location_section(true)
            .display_env_section(true)
            .install()
            .unwrap();

        // tracing_subscriber::fmt()
        //     .with_max_level(tracing::Level::TRACE)
        //     .with_file(true)
        //     .with_line_number(true)
        //     .compact()
        //     .init();

        let layer = tracing_tree::HierarchicalLayer::new(2)
            .with_indent_lines(true)
            .with_targets(true)
            .with_bracketed_fields(true);

        let subscriber = tracing_subscriber::Registry::default().with(layer);
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}
