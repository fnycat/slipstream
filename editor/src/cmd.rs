use futures::{SinkExt, channel::mpsc};

use crate::{error::EditorResult, pages::RoutablePage};

pub enum AppCommand {
    Route(Box<dyn RoutablePage>),
    CenterWindow,
}

#[derive(Clone)]
pub struct AppCommandChannel {
    sender: mpsc::Sender<AppCommand>,
}

impl AppCommandChannel {
    pub fn new(sender: mpsc::Sender<AppCommand>) -> Self {
        Self { sender }
    }

    pub fn try_route(&mut self, page_data: Box<dyn RoutablePage>) -> EditorResult<()> {
        self.sender.try_send(AppCommand::Route(page_data))?;
        Ok(())
    }

    pub async fn route(&mut self, page_data: Box<dyn RoutablePage>) -> EditorResult<()> {
        // Use send to force the channel to be flushed.
        self.sender.send(AppCommand::Route(page_data)).await?;
        Ok(())
    }

    pub fn try_center_window(&mut self) -> EditorResult<()> {
        self.sender.try_send(AppCommand::CenterWindow)?;
        Ok(())
    }

    pub async fn center_window(&mut self) -> EditorResult<()> {
        self.sender.send(AppCommand::CenterWindow).await?;
        Ok(())
    }
}
