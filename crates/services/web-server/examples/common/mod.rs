#![allow(dead_code)] // Shared by multiple example binaries with partial usage per binary.

use reqwest::header::{COOKIE, SET_COOKIE};
use reqwest::Client;
use serde_json::json;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub struct FlowClient {
    client: Client,
    base_url: String,
    auth_cookie: String,
}

impl FlowClient {
    pub async fn login_from_env() -> Result<Self> {
        let base_url = std::env::var("E2BR3_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let email = std::env::var("E2BR3_EXAMPLE_EMAIL")
            .unwrap_or_else(|_| "demo.user@example.com".to_string());
        let pwd = std::env::var("E2BR3_EXAMPLE_PWD")
            .unwrap_or_else(|_| "welcome".to_string());

        let client = reqwest::Client::builder()
            .no_proxy()
            .build()?;

        let login_res = client
            .post(format!("{base_url}/auth/v1/login"))
            .json(&json!({
                "email": email,
                "pwd": pwd,
            }))
            .send()
            .await?;
        if !login_res.status().is_success() {
            let _ = login_res.text().await;
            return Err("login failed; check E2BR3_EXAMPLE_EMAIL / E2BR3_EXAMPLE_PWD".into());
        }
        let auth_cookie = extract_auth_cookie(login_res.headers())
            .ok_or("login succeeded but auth-token cookie not set")?;

        println!("logged in as {}", email);
        Ok(Self {
            client,
            base_url,
            auth_cookie,
        })
    }

    pub fn default_org_id() -> String {
        std::env::var("E2BR3_EXAMPLE_ORG_ID")
            .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".to_string())
    }

    pub async fn create_case(&self, case_prefix: &str) -> Result<(String, String)> {
        let safety_report_id = format!("{}-{}", case_prefix, Uuid::new_v4());
        let org_id = Self::default_org_id();

        let body = json!({
            "data": {
                "status": "draft",
                "organization_id": org_id,
                "safety_report_id": safety_report_id,
            }
        });

        let value = self.post_json("/api/cases", body).await?;
        let case_id = extract_id(&value).ok_or("missing case id in create case response")?;
        Ok((case_id, safety_report_id))
    }

    pub async fn seed_minimum_case_data(
        &self,
        case_id: &str,
        receiver_identifier: &str,
        batch_receiver_identifier: &str,
    ) -> Result<()> {
        self.post_json(
            &format!("/api/cases/{case_id}/message-header"),
            json!({
                "data": {
                    "case_id": case_id,
                    "message_date": "20240101120000",
                    "message_date_format": "204",
                    "message_format_release": "2.0",
                    "message_format_version": "2.1",
                    "message_number": format!("MSG-{}", case_id),
                    "message_receiver_identifier": receiver_identifier,
                    "message_sender_identifier": "DSJP",
                    "batch_receiver_identifier": batch_receiver_identifier,
                    "message_type": "ichicsr"
                }
            }),
        )
        .await?;

        self.post_json(
            &format!("/api/cases/{case_id}/safety-report"),
            json!({
                "data": {
                    "case_id": case_id,
                    "transmission_date": [2024, 15],
                    "report_type": "1",
                    "date_first_received_from_source": [2024, 10],
                    "date_of_most_recent_information": [2024, 15],
                    "fulfil_expedited_criteria": true,
                    "local_criteria_report_type": "1",
                    "combination_product_report_indicator": "false"
                }
            }),
        )
        .await?;

        self.post_json(
            &format!("/api/cases/{case_id}/safety-report/senders"),
            json!({
                "data": {
                    "case_id": case_id,
                    "sender_type": "1",
                    "organization_name": "Example Sender Org"
                }
            }),
        )
        .await?;

        self.post_json(
            &format!("/api/cases/{case_id}/patient"),
            json!({
                "data": {
                    "case_id": case_id,
                    "patient_initials": "PT",
                    "sex": "2",
                    "age_group": "5",
                    "race_code": "C41260",
                    "medical_history_text": "None"
                }
            }),
        )
        .await?;

        self.post_json(
            &format!("/api/cases/{case_id}/reactions"),
            json!({
                "data": {
                    "case_id": case_id,
                    "sequence_number": 1,
                    "primary_source_reaction": "Headache",
                    "serious": false,
                    "outcome": "1"
                }
            }),
        )
        .await?;

        self.post_json(
            &format!("/api/cases/{case_id}/drugs"),
            json!({
                "data": {
                    "case_id": case_id,
                    "sequence_number": 1,
                    "drug_characterization": "1",
                    "medicinal_product": "Drug A"
                }
            }),
        )
        .await?;

        self.post_json(
            &format!("/api/cases/{case_id}/narrative"),
            json!({
                "data": {
                    "case_id": case_id,
                    "case_narrative": "Example narrative"
                }
            }),
        )
        .await?;

        Ok(())
    }

    pub async fn import_xml_file(&self, xml_path: &Path) -> Result<String> {
        let cookie = self.auth_cookie_header()?;
        let xml = fs::read_to_string(xml_path)?;
        let boundary = "X-BOUNDARY-E2BR3-EXAMPLE";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"case.xml\"\r\nContent-Type: application/xml\r\n\r\n{xml}\r\n--{boundary}--\r\n"
        );

        let res = self
            .client
            .post(format!("{}/api/import/xml", self.base_url))
            .header(COOKIE, cookie)
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            return Err(format!("import failed: {status} {text}").into());
        }

        let value: Value = serde_json::from_str(&text)?;
        let case_id = value
            .get("data")
            .and_then(|v| {
                v.get("case_id")
                    .or_else(|| v.get("caseId"))
                    .or_else(|| v.get("id"))
            })
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .ok_or("missing case_id in import response")?;

        Ok(case_id)
    }

    pub async fn validate_case(&self, case_id: &str, profile: &str) -> Result<Value> {
        let path = format!("/api/cases/{case_id}/validation?profile={profile}");
        self.get_json(&path).await
    }

    pub async fn mark_case_validated(&self, case_id: &str) -> Result<()> {
        self.put_json(
            &format!("/api/cases/{case_id}"),
            json!({ "data": { "status": "validated" } }),
        )
        .await?;
        Ok(())
    }

    pub async fn export_xml(&self, case_id: &str) -> Result<String> {
        let cookie = self.auth_cookie_header()?;
        let res = self
            .client
            .get(format!("{}/api/cases/{case_id}/export/xml", self.base_url))
            .header(COOKIE, cookie)
            .send()
            .await?;

        let status = res.status();
        let body = res.text().await?;
        if !status.is_success() {
            return Err(format!("export failed: {status} {body}").into());
        }
        Ok(body)
    }

    pub fn write_export_to_dir(&self, label: &str, xml: &str) -> Result<String> {
        let dir = env::var("E2BR3_EXAMPLE_OUTPUT_DIR").unwrap_or_else(|_| {
            env::var("HOME")
                .map(|home| Path::new(&home).join("Documents"))
                .unwrap_or_else(|_| "/tmp/e2br3_examples".into())
                .to_string_lossy()
                .to_string()
        });
        fs::create_dir_all(&dir)?;
        let file = format!("{dir}/{label}_{}.xml", Uuid::new_v4());
        fs::write(&file, xml)?;
        Ok(file)
    }

    async fn get_json(&self, path: &str) -> Result<Value> {
        let cookie = self.auth_cookie_header()?;
        let res = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .header(COOKIE, cookie)
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            return Err(format!("GET {path} failed: {status} {text}").into());
        }
        Ok(serde_json::from_str(&text)?)
    }

    async fn post_json(&self, path: &str, body: Value) -> Result<Value> {
        let cookie = self.auth_cookie_header()?;
        let res = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .header(COOKIE, cookie)
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            return Err(format!("POST {path} failed: {status} {text}").into());
        }
        Ok(serde_json::from_str(&text)?)
    }

    async fn put_json(&self, path: &str, body: Value) -> Result<Value> {
        let cookie = self.auth_cookie_header()?;
        let res = self
            .client
            .put(format!("{}{}", self.base_url, path))
            .header(COOKIE, cookie)
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            return Err(format!("PUT {path} failed: {status} {text}").into());
        }
        Ok(serde_json::from_str(&text)?)
    }

    fn auth_cookie_header(&self) -> Result<String> {
        Ok(self.auth_cookie.clone())
    }
}

fn extract_id(value: &Value) -> Option<String> {
    value
        .get("data")
        .and_then(|v| v.get("id").or_else(|| v.get("case_id")).or_else(|| v.get("caseId")))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
}

fn extract_auth_cookie(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|cookie| {
            cookie
                .split(';')
                .find(|part| part.trim_start().starts_with("auth-token="))
                .map(|part| part.trim().to_string())
        })
}
