use crate::error::EditorResult;

pub mod info;
pub mod intro;
pub mod panic;
pub mod settings;
pub mod splash;

pub trait RoutablePage: Send {
    fn name(&self) -> &str;
    fn draw(&mut self, ui: &mut egui::Ui) -> EditorResult<()>;

    fn update(&mut self) -> EditorResult<()> {
        Ok(())
    }
}
