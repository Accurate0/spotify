//! `spotify link <account>`: authorizes the server to read an account's recently played tracks.
//! Spotify redirects back to a loopback listener with a code, which the server exchanges for the
//! account's refresh token, so the client secret never leaves the server.

use std::hash::{BuildHasher, RandomState};
use std::time::Duration;

use anyhow::Context;
use cynic::{MutationBuilder, QueryBuilder};
use reqwest::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::client::Client;
use crate::graphql::{
    AccountQuery, AccountVariables, ClientIdQuery, LinkAccount, LinkAccountInput,
    LinkAccountVariables,
};

/// Registered as `http://127.0.0.1:8975/callback` in the Spotify app's redirect URIs.
const PORT: u16 = 8975;
const SCOPE: &str = "user-read-recently-played";
const TIMEOUT: Duration = Duration::from_secs(300);

pub async fn run(client: &Client, account: &str) -> anyhow::Result<()> {
    client
        .run(AccountQuery::build(AccountVariables {
            slug: account.to_owned(),
        }))
        .await?
        .spotify_account
        .with_context(|| format!("account {account} not found"))?;
    let client_id = client
        .run(ClientIdQuery::build(()))
        .await?
        .spotify_client_id
        .context("the server has no Spotify credentials configured")?;

    let redirect_uri = format!("http://127.0.0.1:{PORT}/callback");
    let state = format!("{:016x}", RandomState::new().hash_one(account));
    // show_dialog lets each account pick its own Spotify user rather than reusing the
    // browser's current session silently.
    let authorize = Url::parse_with_params(
        "https://accounts.spotify.com/authorize",
        &[
            ("client_id", client_id.as_str()),
            ("response_type", "code"),
            ("redirect_uri", redirect_uri.as_str()),
            ("scope", SCOPE),
            ("state", state.as_str()),
            ("show_dialog", "true"),
        ],
    )?;

    let listener = TcpListener::bind(("127.0.0.1", PORT))
        .await
        .with_context(|| format!("listening on port {PORT} for Spotify's redirect"))?;
    println!("Sign in as the Spotify user for {account}:\n\n  {authorize}\n");
    open(authorize.as_str());

    let (code, mut stream) = tokio::time::timeout(TIMEOUT, callback(&listener, &state))
        .await
        .context("timed out waiting for Spotify")??;

    let linked = client
        .run(LinkAccount::build(LinkAccountVariables {
            input: LinkAccountInput {
                account: account.to_owned(),
                code,
                redirect_uri,
            },
        }))
        .await;
    let message = match &linked {
        Ok(_) => "Linked. You can close this tab.".to_owned(),
        Err(e) => format!("Linking failed: {e}"),
    };
    respond(&mut stream, "200 OK", &message).await;

    let payload = linked?.spotify_link_account;
    println!(
        "linked {} to Spotify user {}; recent plays are fetched every 30 minutes",
        payload.account.slug, payload.link.spotify_user_id
    );
    Ok(())
}

/// Waits for Spotify's redirect and returns its code, with the connection to answer once the
/// code is exchanged. A bad redirect is answered here; other requests (e.g. favicon) get 404s.
async fn callback(listener: &TcpListener, state: &str) -> anyhow::Result<(String, TcpStream)> {
    loop {
        let (mut stream, _) = listener.accept().await?;
        let mut buf = vec![0; 8192];
        let n = stream.read(&mut buf).await?;
        let request = String::from_utf8_lossy(&buf[..n]);
        let Some(target) = request
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("GET "))
            .and_then(|rest| rest.split_whitespace().next())
        else {
            continue;
        };
        let url = Url::parse(&format!("http://127.0.0.1{target}"))?;
        if url.path() != "/callback" {
            respond(&mut stream, "404 Not Found", "").await;
            continue;
        }

        let param = |name: &str| {
            url.query_pairs()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.into_owned())
        };
        let result = if param("state").as_deref() != Some(state) {
            Err(anyhow::anyhow!("Spotify's redirect had the wrong state"))
        } else if let Some(error) = param("error") {
            Err(anyhow::anyhow!("Spotify declined: {error}"))
        } else {
            param("code").context("Spotify's redirect had no code")
        };

        return match result {
            Ok(code) => Ok((code, stream)),
            Err(e) => {
                respond(&mut stream, "200 OK", &format!("Linking failed: {e}")).await;
                Err(e)
            }
        };
    }
}

async fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

/// Opens `url` in the default browser, best effort; the URL is printed either way.
fn open(url: &str) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(opener)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
