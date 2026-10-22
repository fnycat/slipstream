use slipstream_shared::SlipstreamResult;

use crate::viewer::ViewerPane;

impl ViewerPane {
    pub fn draw_lower_toolbar(&mut self, _ui: &mut egui::Ui) -> SlipstreamResult<()> {
        Ok(())
    }
}
