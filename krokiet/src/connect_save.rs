use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rfd::FileDialog;
use slint::{ComponentHandle, Model, ModelRc};

use crate::connect_rfd::{hide_file_dialog_overlay, show_file_dialog_overlay};
use crate::shared_models::SharedModels;
use crate::{ActiveTab, Callabler, GuiState, MainWindow, SingleMainListModel};

pub(crate) fn connect_save(app: &MainWindow, shared_models: Arc<Mutex<SharedModels>>) {
    let a = app.as_weak();
    app.global::<Callabler>().on_save_results(move || {
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let active_tab = app.global::<GuiState>().get_active_tab();

        show_file_dialog_overlay(&app);

        let weak = a.clone();
        let shared_models = Arc::clone(&shared_models);
        std::thread::spawn(move || {
            let folder = FileDialog::new().pick_folder();

            hide_file_dialog_overlay(&weak);

            if let Some(folder) = folder {
                let folder_str = folder.to_string_lossy().to_string();
                weak.upgrade_in_event_loop(move |app| {
                    let mut models = shared_models.lock().expect("Mutex poisoned");
                    let result = if active_tab == ActiveTab::ExactFilenames {
                        models
                            .shared_exact_filenames_state
                            .as_mut()
                            .ok_or_else(|| "Run an Exact File Names scan before exporting its results".to_string())
                            .and_then(|tool| {
                                tool.retain_name_results(&filename_result_paths(&active_tab.get_tool_model(&app)))
                                    .map_err(|method| format!("Cannot reconcile filename results from a {method:?} scan"))
                            })
                    } else {
                        Ok(())
                    };
                    if let Err(e) = result.and_then(|()| models.save_results(active_tab, &folder_str)) {
                        app.global::<GuiState>().set_info_text(e.into());
                    }
                })
                .expect("Failed to save results");
            }
        });
    });
}

fn filename_result_paths(model: &ModelRc<SingleMainListModel>) -> HashSet<PathBuf> {
    model
        .iter()
        .filter(|row| !row.header_row || row.filled_header_row)
        .map(|row| {
            let name = row
                .val_str
                .row_data(ActiveTab::ExactFilenames.get_str_name_idx())
                .expect("Filename result must contain its name");
            let path = row
                .val_str
                .row_data(ActiveTab::ExactFilenames.get_str_path_idx())
                .expect("Filename result must contain its directory");
            PathBuf::from(path.as_str()).join(name.as_str())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::create_model_from_model_vec;
    use crate::test_common::get_main_list_model;

    #[test]
    fn filename_export_paths_include_reference_and_protected_rows_but_not_empty_headers() {
        let directory = std::env::temp_dir().join("krokiet-visible-name-results");
        let mut header = get_main_list_model();
        header.header_row = true;
        header.val_str = ModelRc::default();
        let mut reference = header.clone();
        reference.filled_header_row = true;
        reference.val_str = create_model_from_model_vec(&["".into(), "reference.txt".into(), directory.to_str().unwrap().into(), "".into()]);
        let mut protected = get_main_list_model();
        protected.protected = true;
        protected.val_str = create_model_from_model_vec(&["".into(), "protected.txt".into(), directory.to_str().unwrap().into(), "".into()]);
        let model = create_model_from_model_vec(&[header, reference, protected]);
        assert_eq!(
            filename_result_paths(&model),
            HashSet::from([directory.join("reference.txt"), directory.join("protected.txt")])
        );
        assert!(filename_result_paths(&ModelRc::default()).is_empty());
    }
}
