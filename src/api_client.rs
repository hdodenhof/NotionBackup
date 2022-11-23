mod auth_header_handler;

use std::error::Error;

use reqwest::Url;
use serde::Deserialize;
use serde_json::json;

use auth_header_handler::AuthHeaderHandler;

pub struct ApiClient {
    http_client: reqwest::blocking::Client,
    base_url: Url,
    auth_header_handler: AuthHeaderHandler,
}

impl ApiClient {
    pub fn new() -> ApiClient {
        ApiClient {
            http_client: reqwest::blocking::Client::new(),
            base_url: Url::parse("https://www.notion.so/api/v3/").unwrap(),
            auth_header_handler: AuthHeaderHandler::new(),
        }
    }

    pub fn login(&self, email: &str, password: &str) -> Result<String, Box<dyn Error>> {
        let req_body = json!({
            "email": email,
            "password": password,
        });

        let resp = self.http_client
            .post(self.build_url("loginWithEmail"))
            .json(&req_body)
            .send()?
            .error_for_status()?;

        let headers = resp.headers();

        let token = self.auth_header_handler.parse_auth_header(headers)?;

        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<(), reqwest::Error> {
        let req_body = json!({});

        self.http_client
            .post(self.build_url("getSpaces"))
            .json(&req_body)
            .headers(self.auth_header_handler.build_auth_header(token))
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn export_space(&self, space_id: &str, token: &str) -> Result<String, Box<dyn Error>> {
        let req_body = json!({
            "task": {
                "eventName": "exportSpace",
                "request": {
                    "spaceId": space_id,
                    "exportOptions": {
                        "exportType": "html",
                        "timeZone": "Europe/Paris",
                        "locale": "en",
                    }
                }
            }
        });

        let resp: ExportSpaceResponse = self.http_client
            .post(self.build_url("enqueueTask"))
            .json(&req_body)
            .headers(self.auth_header_handler.build_auth_header(token))
            .send()?
            .error_for_status()?
            .json()?;

        Ok(resp.task_id)
    }

    pub fn get_task_status(&self, task_id: &str, token: &str) -> Result<TaskStatus, Box<dyn Error>> {
        let req_body = json!({
            "taskIds": [task_id],
        });

        let resp: GetTasksResponse = self.http_client
            .post(self.build_url("getTasks"))
            .json(&req_body)
            .headers(self.auth_header_handler.build_auth_header(token))
            .send()?
            .error_for_status()?
            .json()?;

        let task = resp.results
            .into_iter()
            .filter(|result| result.id == task_id)
            .nth(0)
            .ok_or("Determining task status failed")?;

        Ok(task.status)
    }

    fn build_url(&self, path: &str) -> Url {
        self.base_url.join(path).unwrap()
    }
}

#[derive(Deserialize)]
struct ExportSpaceResponse {
    #[serde(rename(deserialize = "taskId"))]
    task_id: String,
}

#[derive(Deserialize)]
struct GetTasksResponse {
    results: Vec<TaskResult>,
}

#[derive(Deserialize)]
struct TaskResult {
    id: String,
    status: TaskStatus,
}

#[derive(Deserialize)]
pub struct TaskStatus {
    #[serde(rename(deserialize = "type"))]
    pub value: String,
    #[serde(rename(deserialize = "exportURL"), default)]
    pub export_url: String,
}
