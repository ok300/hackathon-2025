use crate::{delete_wiki_post, update_wiki_post, AuthState, PubkyApp, ViewState};

use eframe::egui::{Context, Ui};
use pubky::PubkySession;

pub(crate) fn update(app: &mut PubkyApp, session: &PubkySession, _ctx: &Context, ui: &mut Ui) {
    ui.label("Edit Wiki Page");
    ui.add_space(20.0);

    // Title input field
    ui.label("Title:");
    ui.add_space(10.0);
    ui.add(
        egui::TextEdit::singleline(&mut app.edit_wiki_title)
            .desired_width(f32::INFINITY)
            .hint_text("Enter title (required)"),
    );

    ui.add_space(20.0);

    // Textarea for wiki content
    ui.label("Content:");
    ui.add_space(10.0);

    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut app.edit_wiki_content)
                    .desired_width(f32::INFINITY)
                    .desired_rows(15),
            );
        });

    ui.add_space(20.0);

    ui.horizontal(|ui| {
        if ui.button("Update wiki").clicked() {
            // Validate that title is not empty
            if app.edit_wiki_title.trim().is_empty() {
                log::error!("Title is required");
            } else {
                let session_clone = session.clone();
                let content = app.edit_wiki_content.clone();
                let title = app.edit_wiki_title.clone();
                let page_id = app.selected_wiki_page_id.clone();

                let update_wiki_post_fut = update_wiki_post(&session_clone, &page_id, &title, &content);
                match app.rt.block_on(update_wiki_post_fut) {
                    Ok(_) => {
                        log::info!("Updated wiki post: {}", page_id);
                        // Update the selected content and title to reflect changes
                        app.selected_wiki_content = content;
                        app.selected_wiki_title = title;

                        app.edit_wiki_content.clear();
                        app.edit_wiki_title.clear();
                        app.view_state = ViewState::WikiList;
                        app.needs_refresh = true;
                    }
                    Err(e) => log::error!("Failed to update wiki post: {e}"),
                }
            }
        }

        // Delete button for editing existing page
        if ui.button("Delete page").clicked() {
            let session_clone = session.clone();
            let page_id = app.selected_wiki_page_id.clone();
            let state_clone = app.state.clone();

            let delete_wiki_post_fut = delete_wiki_post(&session_clone, &page_id);
            match app.rt.block_on(delete_wiki_post_fut) {
                Ok(_) => {
                    log::info!("Deleted wiki post: {}", page_id);

                    // Remove from files list
                    if let Ok(mut state) = state_clone.lock() {
                        if let AuthState::Authenticated {
                            ref session,
                            ref mut files,
                            ..
                        } = *state
                        {
                            let own_user_pk = session.info().public_key().to_string();
                            let file_url = format!("pubky://{own_user_pk}/pub/wiki.app/{page_id}");
                            files.retain(|f| f != &file_url);
                        }
                    }
                }
                Err(e) => log::error!("Failed to delete wiki post: {e}"),
            }

            app.edit_wiki_content.clear();
            app.edit_wiki_title.clear();
            app.selected_wiki_page_id.clear();
            app.selected_wiki_content.clear();
            app.selected_wiki_title.clear();
            app.view_state = ViewState::WikiList;
            app.needs_refresh = true;
        }

        if ui.button("Cancel").clicked() {
            app.edit_wiki_content.clear();
            app.edit_wiki_title.clear();
            app.view_state = ViewState::WikiList;
        }
    });
}
