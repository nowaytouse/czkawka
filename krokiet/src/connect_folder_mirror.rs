use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use rfd::FileDialog;
use slint::{CloseRequestResponse, ComponentHandle, ModelRc, VecModel};

use crate::folder_mirror::{Plan, apply, preview};
use crate::model_operations::replace_results_after_removal;
use crate::settings::collect_settings;
use crate::shared_models::SharedModels;
use crate::{ActiveTab, FolderMirrorState, GuiState, MainWindow, MirrorFolder, Translations, flk};

pub(crate) fn connect_folder_mirror(app: &MainWindow, shared_models: Arc<Mutex<SharedModels>>) {
    let plan = Arc::new(Mutex::new(None));
    connect_choose_folder(app, Arc::clone(&plan));
    connect_preview(app, Arc::clone(&plan));
    connect_apply(app, plan, shared_models);

    let weak = app.as_weak();
    app.window().on_close_requested(move || {
        if weak.upgrade().is_some_and(|app| app.global::<FolderMirrorState>().get_busy()) {
            CloseRequestResponse::KeepWindowShown
        } else {
            CloseRequestResponse::HideWindow
        }
    });
}

fn available(app: &MainWindow) -> bool {
    app.global::<GuiState>().get_active_tab() == ActiveTab::ExactFilenames
        && !app.get_scanning()
        && !app.get_processing()
        && !app.global::<GuiState>().get_file_dialog_open()
        && !app.global::<FolderMirrorState>().get_busy()
}

fn clear_preview(app: &MainWindow, plan: &Mutex<Option<Plan>>) {
    *plan.lock().expect("Folder mirror plan mutex poisoned") = None;
    let state = app.global::<FolderMirrorState>();
    state.set_can_apply(false);
    state.set_preview_rows(ModelRc::default());
    state.set_status_text("".into());
}

fn connect_choose_folder(app: &MainWindow, plan: Arc<Mutex<Option<Plan>>>) {
    let weak = app.as_weak();
    app.global::<FolderMirrorState>().on_choose_folder(move |folder| {
        let Some(app) = weak.upgrade().filter(available) else {
            return;
        };
        let state = app.global::<FolderMirrorState>();
        let translations = app.global::<Translations>();
        let (current_path, title) = match folder {
            MirrorFolder::Target => (state.get_target_path(), translations.get_folder_mirror_target_text()),
            MirrorFolder::Reference => (state.get_reference_path(), translations.get_folder_mirror_reference_text()),
            MirrorFolder::Archive => (state.get_archive_path(), translations.get_folder_mirror_archive_text()),
        };
        let plan = Arc::clone(&plan);
        background(
            &app,
            true,
            move || {
                let dialog = FileDialog::new().set_title(title.as_str());
                if current_path.is_empty() {
                    dialog.pick_folder()
                } else {
                    dialog.set_directory(current_path.as_str()).pick_folder()
                }
            },
            move |app, picked| {
                let Some(picked) = picked else {
                    return;
                };
                clear_preview(app, &plan);
                let state = app.global::<FolderMirrorState>();
                let path = picked.to_str();
                let display_path = path.unwrap_or_default().into();
                match folder {
                    MirrorFolder::Target => state.set_target_path(display_path),
                    MirrorFolder::Reference => state.set_reference_path(display_path),
                    MirrorFolder::Archive => state.set_archive_path(display_path),
                }
                if path.is_none() {
                    state.set_status_text(flk!("folder_mirror_path_encoding").into());
                }
            },
        );
    });
}

fn connect_preview(app: &MainWindow, plan: Arc<Mutex<Option<Plan>>>) {
    let weak = app.as_weak();
    app.global::<FolderMirrorState>().on_preview(move || {
        let Some(app) = weak.upgrade().filter(available) else {
            return;
        };
        let state = app.global::<FolderMirrorState>();
        let paths = [state.get_target_path(), state.get_reference_path(), state.get_archive_path()];
        if paths.iter().any(|path| path.is_empty()) {
            return;
        }
        let [target, reference, archive] = paths.map(|path| PathBuf::from(path.as_str()));
        clear_preview(&app, &plan);
        state.set_status_text(flk!("folder_mirror_previewing").into());
        let plan = Arc::clone(&plan);
        background(
            &app,
            false,
            move || {
                preview(&target, &reference, &archive).map(|next| {
                    let rows = next.preview_rows();
                    let summary = next.summary();
                    (next, rows, summary)
                })
            },
            move |app, result| {
                let state = app.global::<FolderMirrorState>();
                match result {
                    Ok((next, rows, summary)) => {
                        let can_apply = next.has_changes();
                        *plan.lock().expect("Folder mirror plan mutex poisoned") = Some(next);
                        state.set_preview_rows(ModelRc::new(VecModel::from(rows)));
                        state.set_status_text(summary.into());
                        state.set_can_apply(can_apply);
                    }
                    Err(error) => state.set_status_text(flk!("folder_mirror_preview_error", error = error.to_string()).into()),
                }
            },
        );
    });
}

fn connect_apply(app: &MainWindow, plan: Arc<Mutex<Option<Plan>>>, shared_models: Arc<Mutex<SharedModels>>) {
    let weak = app.as_weak();
    app.global::<FolderMirrorState>().on_apply(move || {
        let Some(app) = weak.upgrade().filter(available) else {
            return;
        };
        let state = app.global::<FolderMirrorState>();
        if !state.get_can_apply() {
            return;
        }
        let Some(next) = plan.lock().expect("Folder mirror plan mutex poisoned").take() else {
            state.set_can_apply(false);
            return;
        };
        state.set_can_apply(false);
        state.set_status_text(flk!("folder_mirror_applying").into());
        let reference_dirs = collect_settings(&app).included_paths_referenced;
        let shared_models = Arc::clone(&shared_models);
        background(
            &app,
            false,
            move || apply(&next, &reference_dirs),
            move |app, result| {
                // A failed apply can still have moved files before it stopped.
                shared_models.lock().expect("Shared models mutex poisoned").shared_exact_filenames_state = None;
                replace_results_after_removal(app, ActiveTab::ExactFilenames, Vec::new());
                let state = app.global::<FolderMirrorState>();
                state.set_preview_rows(ModelRc::default());
                let message = match result {
                    Ok(report) => flk!("folder_mirror_applied", report = report.summary()),
                    Err(error) => flk!("folder_mirror_apply_error", error = error.to_string()),
                };
                state.set_status_text(message.into());
            },
        );
    });
}

fn background<T: Send + 'static>(app: &MainWindow, file_dialog: bool, work: impl FnOnce() -> T + Send + 'static, complete: impl FnOnce(&MainWindow, T) + Send + 'static) {
    let weak = app.as_weak();
    app.global::<FolderMirrorState>().set_busy(true);
    app.set_processing(true);
    if file_dialog {
        app.global::<GuiState>().set_file_dialog_open(true);
    }
    if let Err(error) = thread::Builder::new().name("folder-mirror".into()).spawn(move || {
        let result = work();
        if let Err(error) = weak.upgrade_in_event_loop(move |app| {
            complete(&app, result);
            finish_background(&app, file_dialog);
        }) {
            log::warn!("Could not deliver folder mirror result: {error}");
        }
    }) {
        finish_background(app, file_dialog);
        app.global::<FolderMirrorState>()
            .set_status_text(flk!("folder_mirror_worker_error", error = error.to_string()).into());
    }
}

fn finish_background(app: &MainWindow, file_dialog: bool) {
    if file_dialog {
        app.global::<GuiState>().set_file_dialog_open(false);
    }
    app.set_processing(false);
    app.global::<FolderMirrorState>().set_busy(false);
}
