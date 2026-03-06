use std::collections::BTreeMap;
use std::default::Default;

use czkawka_core::common::cache::{load_cache_from_file_generalized_by_path, load_cache_from_file_generalized_by_size, save_cache_to_file_generalized};
use czkawka_core::common::config_cache_path::get_config_cache_path;
use czkawka_core::common::model::HashType;
use czkawka_core::helpers::messages::{MessageLimit, Messages};
use czkawka_core::re_exported::HashAlg;
use czkawka_core::tools::duplicate::DuplicateEntry;
use czkawka_core::tools::duplicate::core::get_duplicate_cache_file;
use czkawka_core::tools::similar_images::core::get_similar_images_cache_file;
use czkawka_core::tools::similar_videos::core::get_similar_videos_cache_file;
use czkawka_core::tools::similar_videos::{DEFAULT_CROP_DETECT, DEFAULT_SKIP_FORWARD_AMOUNT, DEFAULT_VID_HASH_DURATION};
use gtk4::prelude::*;
use image::imageops::FilterType;
use log::error;

use crate::flg;
use crate::helpers::async_dialog::alert_confirm;
use crate::gui_structs::gui_data::GuiData;
use crate::saving_loading::{load_configuration, reset_configuration, save_configuration};

pub(crate) fn connect_settings(gui_data: &GuiData) {
    // Connect scale
    {
        let label_restart_needed = gui_data.settings.label_restart_needed.clone();
        gui_data.settings.scale_settings_number_of_threads.connect_value_changed(move |_| {
            if label_restart_needed.label().is_empty() {
                label_restart_needed.set_label(&flg!("settings_label_restart"));
            }
        });
    }
    // Connect button settings
    {
        let button_settings = gui_data.header.button_settings.clone();
        let window_settings = gui_data.settings.window_settings.clone();
        button_settings.connect_clicked(move |_| {
            window_settings.set_visible(true);
        });

        let window_settings = gui_data.settings.window_settings.clone();

        window_settings.connect_close_request(move |window| {
            window.set_visible(false);
            glib::Propagation::Stop
        });
    }

    // Connect save configuration button
    {
        let upper_notebook = gui_data.upper_notebook.clone();
        let settings = gui_data.settings.clone();
        let main_notebook = gui_data.main_notebook.clone();
        let text_view_errors = gui_data.text_view_errors.clone();
        let button_settings_save_configuration = gui_data.settings.button_settings_save_configuration.clone();
        button_settings_save_configuration.connect_clicked(move |_| {
            save_configuration(true, &upper_notebook, &main_notebook, &settings, &text_view_errors);
        });
    }
    // Connect load configuration button
    {
        let upper_notebook = gui_data.upper_notebook.clone();
        let settings = gui_data.settings.clone();
        let main_notebook = gui_data.main_notebook.clone();
        let text_view_errors = gui_data.text_view_errors.clone();
        let button_settings_load_configuration = gui_data.settings.button_settings_load_configuration.clone();
        let scrolled_window_errors = gui_data.scrolled_window_errors.clone();
        button_settings_load_configuration.connect_clicked(move |_| {
            load_configuration(true, &upper_notebook, &main_notebook, &settings, &text_view_errors, &scrolled_window_errors, None);
        });
    }
    // Connect reset configuration button
    {
        let upper_notebook = gui_data.upper_notebook.clone();
        let settings = gui_data.settings.clone();
        let main_notebook = gui_data.main_notebook.clone();
        let text_view_errors = gui_data.text_view_errors.clone();
        let button_settings_reset_configuration = gui_data.settings.button_settings_reset_configuration.clone();
        button_settings_reset_configuration.connect_clicked(move |_| {
            reset_configuration(true, &upper_notebook, &main_notebook, &settings, &text_view_errors);
        });
    }
    // Connect button for opening cache
    {
        let button_settings_open_cache_folder = gui_data.settings.button_settings_open_cache_folder.clone();
        button_settings_open_cache_folder.connect_clicked(move |_| {
            if let Some(config_cache_path) = get_config_cache_path() {
                if let Err(e) = open::that(&config_cache_path.cache_folder) {
                    error!("Failed to open config folder \"{}\", reason {e}", config_cache_path.cache_folder.to_string_lossy());
                }
            } else {
                error!("Failed to get cache folder path");
            }
        });
    }
    // Connect button for opening settings
    {
        let button_settings_open_settings_folder = gui_data.settings.button_settings_open_settings_folder.clone();
        button_settings_open_settings_folder.connect_clicked(move |_| {
            if let Some(config_cache_path) = get_config_cache_path() {
                if let Err(e) = open::that(&config_cache_path.config_folder) {
                    error!("Failed to open config folder \"{}\", reason {e}", config_cache_path.config_folder.to_string_lossy());
                }
            } else {
                error!("Failed to get settings folder path");
            }
        });
    }
    // Connect clear cache methods
    {
        {
            let button_settings_duplicates_clear_cache = gui_data.settings.button_settings_duplicates_clear_cache.clone();
            let settings_window = gui_data.settings.window_settings.clone();
            let text_view_errors = gui_data.text_view_errors.clone();
            let entry_settings_cache_file_minimal_size = gui_data.settings.entry_settings_cache_file_minimal_size.clone();

            button_settings_duplicates_clear_cache.connect_clicked(move |_| {
                let title = flg!("cache_clear_duplicates_title");
                let detail = cache_clear_detail();
                let settings_window = settings_window.clone();
                let text_view_errors = text_view_errors.clone();
                let entry_settings_cache_file_minimal_size = entry_settings_cache_file_minimal_size.clone();

                glib::MainContext::default().spawn_local(async move {
                    if alert_confirm(&settings_window, &title, &detail).await {
                        let mut messages: Messages = Messages::new();
                        for use_prehash in [true, false] {
                            for type_of_hash in [HashType::Xxh3, HashType::Blake3, HashType::Crc32] {
                                let file_name = get_duplicate_cache_file(type_of_hash, use_prehash);
                                let (mut messages, loaded_items) = load_cache_from_file_generalized_by_size::<DuplicateEntry>(&file_name, true, &Default::default());

                                if let Some(cache_entries) = loaded_items {
                                    let mut hashmap_to_save: BTreeMap<String, DuplicateEntry> = Default::default();
                                    for (_, vec_file_entry) in cache_entries {
                                        for file_entry in vec_file_entry {
                                            hashmap_to_save.insert(file_entry.path.to_string_lossy().to_string(), file_entry);
                                        }
                                    }

                                    let minimal_cache_size = entry_settings_cache_file_minimal_size.text().as_str().parse::<u64>().unwrap_or(2 * 1024 * 1024);

                                    let save_messages = save_cache_to_file_generalized(&file_name, &hashmap_to_save, false, minimal_cache_size);
                                    messages.extend_with_another_messages(save_messages);
                                }
                            }

                            messages.messages.push(flg!("cache_properly_cleared"));
                            text_view_errors.buffer().set_text(messages.create_messages_text(MessageLimit::NoLimit).as_str());
                        }
                    }
                });
            });
        }
        {
            let button_settings_similar_images_clear_cache = gui_data.settings.button_settings_similar_images_clear_cache.clone();
            let settings_window = gui_data.settings.window_settings.clone();
            let text_view_errors = gui_data.text_view_errors.clone();

            button_settings_similar_images_clear_cache.connect_clicked(move |_| {
                let title = flg!("cache_clear_similar_images_title");
                let detail = cache_clear_detail();
                let settings_window = settings_window.clone();
                let text_view_errors = text_view_errors.clone();

                glib::MainContext::default().spawn_local(async move {
                    if alert_confirm(&settings_window, &title, &detail).await {
                        let mut messages: Messages = Messages::new();
                        for hash_size in [8, 16, 32, 64] {
                            for image_filter in [FilterType::Lanczos3, FilterType::CatmullRom, FilterType::Gaussian, FilterType::Nearest, FilterType::Triangle] {
                                for hash_alg in [HashAlg::Blockhash, HashAlg::Gradient, HashAlg::DoubleGradient, HashAlg::VertGradient, HashAlg::Mean, HashAlg::Median] {
                                    let file_name = get_similar_images_cache_file(hash_size, hash_alg, image_filter);
                                    let (mut messages, loaded_items) =
                                        load_cache_from_file_generalized_by_path::<czkawka_core::tools::similar_images::ImagesEntry>(&file_name, true, &Default::default());

                                    if let Some(cache_entries) = loaded_items {
                                        let save_messages = save_cache_to_file_generalized(&file_name, &cache_entries, false, 0);
                                        messages.extend_with_another_messages(save_messages);
                                    }
                                }
                            }
                        }

                        messages.messages.push(flg!("cache_properly_cleared"));
                        text_view_errors.buffer().set_text(messages.create_messages_text(MessageLimit::NoLimit).as_str());
                    }
                });
            });
        }
        {
            let button_settings_similar_videos_clear_cache = gui_data.settings.button_settings_similar_videos_clear_cache.clone();
            let settings_window = gui_data.settings.window_settings.clone();
            let text_view_errors = gui_data.text_view_errors.clone();

            button_settings_similar_videos_clear_cache.connect_clicked(move |_| {
                let title = flg!("cache_clear_similar_videos_title");
                let detail = cache_clear_detail();
                let settings_window = settings_window.clone();
                let text_view_errors = text_view_errors.clone();

                glib::MainContext::default().spawn_local(async move {
                    if alert_confirm(&settings_window, &title, &detail).await {
                        let file_name = get_similar_videos_cache_file(DEFAULT_SKIP_FORWARD_AMOUNT, DEFAULT_VID_HASH_DURATION, DEFAULT_CROP_DETECT);
                        let (mut messages, loaded_items) =
                            load_cache_from_file_generalized_by_path::<czkawka_core::tools::similar_videos::VideosEntry>(&file_name, true, &Default::default());

                        let mut messages = if let Some(cache_entries) = loaded_items {
                            let save_messages = save_cache_to_file_generalized(&file_name, &cache_entries, false, 0);
                            messages.extend_with_another_messages(save_messages);
                            messages
                        } else {
                            messages
                        };

                        messages.messages.push(flg!("cache_properly_cleared"));
                        text_view_errors.buffer().set_text(messages.create_messages_text(MessageLimit::NoLimit).as_str());
                    }
                });
            });
        }
    }
}

fn cache_clear_detail() -> String {
    format!("{}\n{}\n{}\n{}", flg!("cache_clear_message_label_1"), flg!("cache_clear_message_label_2"), flg!("cache_clear_message_label_3"), flg!("cache_clear_message_label_4"))
}
