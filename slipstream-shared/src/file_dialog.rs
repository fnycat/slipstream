#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FileDialogKind {
    PickSingle,
    Save
}

pub fn pick_file_dialog<F, Fut>(kind: FileDialogKind, callback: F)
where
    F: FnOnce(rfd::FileHandle) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send
{
    let future = async move {
        let file = rfd::AsyncFileDialog::new();
        let file = match kind {
            FileDialogKind::PickSingle => file.pick_file().await,
            FileDialogKind::Save => file.save_file().await
        };

        if let Some(file) = file {
            callback(file).await;
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(move || {
            futures::executor::block_on(future);
        });
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(future);
    }
}
