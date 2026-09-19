egui_phosphor::subset! {
    pub mod icons {
        use regular::{
            FOLDER, FOLDER_OPEN, FOLDER_MINUS, FOLDER_PLUS, X, INFO, MINUS, SQUARE, FOLDER_DASHED, BONE,
            QUESTION_MARK, FILE, CUBE, POLYGON, ARROW_ELBOW_RIGHT, MOON, GEAR_FINE, GITHUB_LOGO, SUN, POWER,
            FILE_CODE
        };
        use fill::{FOLDER, BONE};
    }
}

macro_rules! reg_icon {
    ($icon:ident) => {
        // $crate::icons::icons::regular::rich($crate::icons::icons::regular::$icon)
        egui::RichText::new($crate::icons::icons::regular::$icon)
    };
}

macro_rules! fill_icon {
    ($icon:ident) => {
        $crate::icons::icons::fill::rich($crate::icons::icons::fill::$icon)
    };
}
