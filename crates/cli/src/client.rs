use anyhow::{Context, bail};
use cynic::{GraphQlResponse, Operation};
use reqwest::multipart::{Form, Part};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct Client {
    http: reqwest::Client,
    endpoint: String,
    token: Option<String>,
}

impl Client {
    pub fn new(base_url: &str, token: Option<String>) -> anyhow::Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder()
                .user_agent(concat!("spotify-cli/", env!("CARGO_PKG_VERSION")))
                .build()?,
            endpoint: format!("{}/v1/graphql", base_url.trim_end_matches('/')),
            token,
        })
    }

    pub async fn run<Q, V>(&self, operation: Operation<Q, V>) -> anyhow::Result<Q>
    where
        Q: DeserializeOwned,
        V: Serialize,
    {
        let request = self.http.post(&self.endpoint).json(&operation);
        self.send(request).await
    }

    /// Sends an operation whose `file` variable is an `Upload`, as a GraphQL multipart request.
    pub async fn upload<Q, V>(
        &self,
        operation: Operation<Q, V>,
        file_name: &str,
        bytes: Vec<u8>,
    ) -> anyhow::Result<Q>
    where
        Q: DeserializeOwned,
        V: Serialize,
    {
        let form = Form::new()
            .text("operations", serde_json::to_string(&operation)?)
            .text("map", r#"{"0":["variables.file"]}"#)
            .part(
                "0",
                Part::bytes(bytes)
                    .file_name(file_name.to_owned())
                    .mime_str("application/json")?,
            );

        let request = self.http.post(&self.endpoint).multipart(form);
        self.send(request).await
    }

    async fn send<Q: DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> anyhow::Result<Q> {
        let request = match &self.token {
            Some(token) => request.bearer_auth(token),
            None => request,
        };

        let response = request
            .send()
            .await
            .with_context(|| format!("request to {} failed", self.endpoint))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("{status}: {body}");
        }

        let response: GraphQlResponse<Q> = response.json().await.context("invalid response")?;
        if let Some(errors) = response.errors.filter(|e| !e.is_empty()) {
            let messages: Vec<_> = errors.into_iter().map(|e| e.message).collect();
            bail!("{}", messages.join("; "));
        }

        response.data.context("response contained no data")
    }
}
