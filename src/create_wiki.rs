use crate::{create_wiki_post, utils::extract_title, AuthState, PubkyApp, ViewState};

use eframe::egui::{Context, Ui};
use pubky::PubkySession;

pub(crate) fn update(app: &mut PubkyApp, session: &PubkySession, _ctx: &Context, ui: &mut Ui) {
    ui.heading(egui::RichText::new("Create New Wiki Page").size(20.0));
    ui.add_space(30.0);

    // Textarea for wiki content with better frame
    ui.label(egui::RichText::new("Content (Markdown):").size(14.0).strong());
    ui.add_space(10.0);

    egui::Frame::NONE
        .fill(egui::Color32::from_gray(250))
        .inner_margin(8.0)
        .corner_radius(4.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(200)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut app.edit_wiki_content)
                            .desired_width(f32::INFINITY)
                            .desired_rows(15)
                            .font(egui::TextStyle::Monospace),
                    );
                });
        });

    ui.add_space(25.0);

    ui.horizontal(|ui| {
        // Save button for creating new page
        let save_btn = egui::Button::new(
            egui::RichText::new("💾 Save wiki").size(14.0)
        ).min_size(egui::vec2(120.0, 32.0));
        
        if ui.add(save_btn).clicked() {
            let session_clone = session.clone();
            let content = app.edit_wiki_content.clone();
            let state_clone = app.state.clone();

            let create_wiki_post_fut = create_wiki_post(&session_clone, &content);
            match app.rt.block_on(create_wiki_post_fut) {
                Ok(wiki_page_path) => {
                    log::info!("Created wiki post at: {}", wiki_page_path);

                    // Convert path to pubky URL format for the file_cache list
                    if let Ok(mut state) = state_clone.lock() {
                        if let AuthState::Authenticated {
                            ref session,
                            ref mut file_cache,
                            ..
                        } = *state
                        {
                            let own_user_pk = session.info().public_key().to_string();
                            let file_url = format!("pubky://{own_user_pk}{wiki_page_path}");
                            let file_title = extract_title(&content);
                            file_cache.insert(file_url, file_title.into());
                        }
                    }
                }
                Err(e) => log::error!("Failed to create wiki post: {e}"),
            }

            app.edit_wiki_content.clear();
            app.view_state = ViewState::WikiList;
        }

        ui.add_space(8.0);
        
        let cancel_btn = egui::Button::new(
            egui::RichText::new("Cancel").size(14.0)
        ).min_size(egui::vec2(100.0, 32.0));
        
        if ui.add(cancel_btn).clicked() {
            app.edit_wiki_content.clear();
            app.view_state = ViewState::WikiList;
        }
    });
}
