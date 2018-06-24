use failure::Error;
use okta::OktaLinks;
use okta::OktaLinks::Multi;
use okta::OktaLinks::Single;
use okta::OktaLoginResponse;
use reqwest;
use reqwest::header::{Accept, ContentType, Cookie};
use serde_json;
use std::collections::HashMap;
use std::fmt;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "factorType")]
pub enum Factor {
    #[serde(rename_all = "camelCase")]
    Push {
        id: String,
        provider: FactorProvider,
        status: Option<FactorStatus>,
        #[serde(rename = "_links")]
        links: HashMap<String, OktaLinks>,
    },
    #[serde(rename_all = "camelCase")]
    Sms {
        id: String,
        provider: FactorProvider,
        status: Option<FactorStatus>,
        profile: SmsFactorProfile,
        #[serde(rename = "_links")]
        links: HashMap<String, OktaLinks>,
    },
    #[serde(rename_all = "camelCase")]
    Call {
        id: String,
        provider: FactorProvider,
        status: Option<FactorStatus>,
        profile: CallFactorProfile,
        #[serde(rename = "_links")]
        links: HashMap<String, OktaLinks>,
    },
    #[serde(rename = "token", rename_all = "camelCase")]
    Token {
        id: String,
        provider: FactorProvider,
        status: Option<FactorStatus>,
        profile: TokenFactorProfile,
        #[serde(rename = "_links")]
        links: HashMap<String, OktaLinks>,
        verify: Option<FactorVerification>,
    },
    #[serde(rename = "token:software:totp", rename_all = "camelCase")]
    Totp {
        id: String,
        provider: FactorProvider,
        status: Option<FactorStatus>,
        profile: TokenFactorProfile,
        #[serde(rename = "_links")]
        links: HashMap<String, OktaLinks>,
    },
    #[serde(rename = "token:hardware", rename_all = "camelCase")]
    Hotp {
        id: String,
        provider: FactorProvider,
        status: Option<FactorStatus>,
        profile: TokenFactorProfile,
        #[serde(rename = "_links")]
        links: HashMap<String, OktaLinks>,
        verify: Option<FactorVerification>,
    },
    #[serde(rename_all = "camelCase")]
    Question {
        id: String,
        provider: FactorProvider,
        status: Option<FactorStatus>,
        profile: QuestionFactorProfile,
        #[serde(rename = "_links")]
        links: HashMap<String, OktaLinks>,
    },
    #[serde(rename_all = "camelCase")]
    Web {
        id: String,
        provider: FactorProvider,
        status: Option<FactorStatus>,
        profile: WebFactorProfile,
        #[serde(rename = "_links")]
        links: HashMap<String, OktaLinks>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactorProvider {
    Okta,
    Rsa,
    Symantec,
    Google,
    Duo,
    Yubico,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactorStatus {
    NotSetup,
    PendingActivation,
    Enrolled,
    Active,
    Inactive,
    Expired,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FactorVerification {
    pass_code: String,
    next_pass_code: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmsFactorProfile {
    phone_number: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallFactorProfile {
    phone_number: String,
    phone_extension: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuestionFactorProfile {
    question: String,
    question_text: String,
    answer: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenFactorProfile {
    credential_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebFactorProfile {
    credential_id: String,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(untagged)]
pub enum FactorVerificationRequest {
    #[serde(rename_all = "camelCase")]
    Question { answer: String },
    #[serde(rename_all = "camelCase")]
    Sms {
        state_token: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pass_code: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Call { pass_code: Option<String> },
    #[serde(rename_all = "camelCase")]
    Totp { pass_code: String },
    #[serde(rename_all = "camelCase")]
    Token { pass_code: String },
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Factor::Push { .. } => write!(f, "Okta Verify Push"),
            Factor::Sms { ref profile, .. } => write!(f, "Okta SMS to {}", profile.phone_number),
            Factor::Call { ref profile, .. } => write!(f, "Okta Call to {}", profile.phone_number),
            Factor::Token { .. } => write!(f, "Okta One-time Password"),
            Factor::Totp { .. } => write!(f, "Okta Time-based One-time Password"),
            Factor::Hotp { .. } => write!(f, "Okta Hardware One-time Password"),
            Factor::Question { ref profile, .. } => write!(f, "Question: {}", profile.question),
            Factor::Web { .. } => write!(f, "Okta Web"),
        }
    }
}

impl Factor {
    pub fn verify(&self, request: FactorVerificationRequest) -> Result<OktaLoginResponse, Error> {
        let client = reqwest::Client::new();

        match *self {
            Factor::Sms { ref links, .. } => {
                let url = match links.get("verify").unwrap() {
                    Single(ref link) => link.href.clone(),
                    Multi(ref links) => links.first().unwrap().href.clone(),
                };

                let resp = client
                    .post(url)
                    .json(&request)
                    .header(ContentType::json())
                    .header(Accept::json())
                    .send()?
                    .error_for_status()?
                    .text()?;

                trace!("Response: {}", resp);

                serde_json::from_str::<OktaLoginResponse>(&resp).map_err(|e| e.into())
            }
            _ => {
                // TODO
                bail!("Unsupported MFA method")
            }
        }
    }
}
