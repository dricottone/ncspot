use cursive::traits::Resizable;
use cursive::view::Nameable;
use cursive::views::*;
use cursive::Cursive;

use crate::fs::cache_path;
use crate::spotify::Spotify;
use crate::ui::create_cursive;

use librespot_core::authentication::Credentials as RespotCredentials;
use librespot_core::cache::Cache;
use librespot_protocol::authentication::AuthenticationType;

/// Get credentials for use with librespot. This first tries to get cached credentials. If no cached
/// credentials are available, it will either try to get them from the user configured commands, or
/// if that fails, it will prompt the user on stdout.
pub fn get_credentials() -> Result<RespotCredentials, String> {
    let mut credentials = {
        let cache = Cache::new(Some(cache_path("librespot")), None, None, None)
            .expect("Could not create librespot cache");
        let cached_credentials = cache.credentials();
        match cached_credentials {
            Some(c) => c,
            None => credentials_prompt(None)?,
        }
    };

    while let Err(error) = Spotify::test_credentials(credentials.clone()) {
        let error_msg = format!("{error}");
        credentials = credentials_prompt(Some(error_msg))?;
    }
    Ok(credentials)
}

fn credentials_prompt(error_message: Option<String>) -> Result<RespotCredentials, String> {
    if let Some(message) = error_message {
        let mut siv = create_cursive().unwrap();
        let dialog = cursive::views::Dialog::around(cursive::views::TextView::new(format!(
            "Connection error:\n{message}"
        )))
        .button("Ok", |s| s.quit());
        siv.add_layer(dialog);
        siv.run();
    }

    create_credentials()
}

pub fn create_credentials() -> Result<RespotCredentials, String> {
    let mut login_cursive = create_cursive().unwrap();
    let info_buf = TextContent::new("Please login to Spotify\n");
    let info_view = Dialog::around(TextView::new_with_content(info_buf))
        .button("Login", move |s| {
            let login_view = Dialog::new()
                .title("Spotify login")
                .content(
                    ListView::new()
                        .child(
                            "Username",
                            EditView::new().with_name("spotify_user").fixed_width(18),
                        )
                        .child(
                            "Password",
                            EditView::new()
                                .secret()
                                .with_name("spotify_password")
                                .fixed_width(18),
                        ),
                )
                .button("Login", |s| {
                    let username = s
                        .call_on_name("spotify_user", |view: &mut EditView| view.get_content())
                        .unwrap()
                        .to_string();
                    let auth_data = s
                        .call_on_name("spotify_password", |view: &mut EditView| view.get_content())
                        .unwrap()
                        .to_string()
                        .as_bytes()
                        .to_vec();
                    s.set_user_data::<Result<RespotCredentials, String>>(Ok(RespotCredentials {
                        username,
                        auth_type: AuthenticationType::AUTHENTICATION_USER_PASS,
                        auth_data,
                    }));
                    s.quit();
                })
                .button("Quit", Cursive::quit);
            s.pop_layer();
            s.add_layer(login_view);
        })
        .button("Quit", Cursive::quit);

    login_cursive.add_layer(info_view);
    login_cursive.run();

    login_cursive
        .user_data()
        .cloned()
        .unwrap_or_else(|| Err("Didn't obtain any credentials".to_string()))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub credentials: RespotCredentials,
    pub error: Option<String>,
}
