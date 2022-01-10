/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::borrow::Cow;

use actix_http::StatusCode;
use actix_web::{
    http::header, web, HttpResponse, HttpResponseBuilder, Responder, ResponseError,
};
use lazy_static::lazy_static;
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use super::get_uuid;
use crate::errors::*;
use crate::pages::errors::ErrorPage;
use crate::AppData;

/* Workflow:
 * 1. Show two buttons:
 *    - like
 *    - dislike
 * 2. User clicks one of the two buttons > post req to server > server returns feedback UUID
 *      It's possible that the user doesn't want to type a message or might forget to type a
 *      message
 * 3. Show message box to collect descriptive feedback > post req with UUID from server
 */

pub const URL_MAX_LENGTH: usize = 2048;

pub mod routes {
    pub struct Feedback {
        pub rating: &'static str,
        pub scope: &'static str,
        pub rating_form: &'static str,
    }

    impl Feedback {
        pub const fn new() -> Feedback {
            let rating = "/api/v1/feedback/{campaign_id}/rating";
            let rating_form = "/api/v1/feedback/{campaign_id}/rating-form";
            let scope = "/api/v1/feedback";
            Feedback {
                rating,
                scope,
                rating_form,
            }
        }

        pub fn add_feedback_route(&self, campaign_id: &str) -> String {
            self.rating.replace("{campaign_id}", &campaign_id)
        }

        pub fn add_feedback_form_route(&self, campaign_id: &str) -> String {
            self.rating_form.replace("{campaign_id}", &campaign_id)
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    let cors = actix_cors::Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["POST"])
        .allow_any_header()
        .max_age(3600)
        .send_wildcard();

    cfg.service(
        actix_web::Scope::new(crate::V1_API_ROUTES.feedback.scope)
            .wrap(cors)
            .service(rating)
            .service(rating_form),
    );
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RatingReq {
    pub helpful: bool,
    pub description: String,
    pub page_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RatingResp {
    pub uuid: String,
}

pub mod runners {
    use super::*;

    pub async fn rating(
        campaign_id: &Uuid,
        payload: &RatingReq,
        data: &AppData,
    ) -> ServiceResult<RatingResp> {
        if payload.page_url.len() > URL_MAX_LENGTH {
            return Err(ServiceError::URLTooLong);
        }
        Url::parse(&payload.page_url)?;

        let now = OffsetDateTime::now_utc();

        let res = sqlx::query!(
            "
       INSERT INTO kaizen_campaign_pages (campaign_id, page_url) 
       VALUES ($1, $2)",
            &campaign_id,
            &payload.page_url
        )
        .execute(&data.db)
        .await;

        if let Err(sqlx::Error::Database(err)) = res {
            if err.code() != Some(Cow::from("23505"))
                || !err.message().contains("kaizen_campaign_pages_page_url_key")
            {
                return Err(sqlx::Error::Database(err).into());
            }
        }

        let mut uuid;

        loop {
            uuid = get_uuid();

            let res=    sqlx::query!(
                "INSERT INTO 
                kaizen_feedbacks (helpful , description, uuid, campaign_id, time, page_url) 
            VALUES ($1, $2, $3, $4, $5, 
                 (SELECT ID from kaizen_campaign_pages WHERE page_url = $6))",
                &payload.helpful,
                &payload.description,
                &uuid,
                &campaign_id,
                &now,
                &payload.page_url,
            )
            .execute(&data.db)
            .await;

            if res.is_ok() {
                break;
            } else if let Err(sqlx::Error::Database(err)) = res {
                if err.code() == Some(Cow::from("23505"))
                    && err.message().contains("kaizen_campaign_uuid_key")
                {
                    continue;
                } else {
                    return Err(sqlx::Error::Database(err).into());
                }
            }
        }

        let resp = RatingResp {
            uuid: uuid.to_string(),
        };

        Ok(resp)
    }
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.feedback.rating.strip_prefix(crate::V1_API_ROUTES.feedback.scope).unwrap()"
)]
pub async fn rating(
    payload: web::Json<RatingReq>,
    path: web::Path<String>,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let payload = payload.into_inner();
    let path = path.into_inner();
    let campaign_id = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    let resp = runners::rating(&campaign_id, &payload, &data).await?;

    Ok(HttpResponse::Ok().json(resp))
}

#[derive(Clone, TemplateOnce)]
#[template(path = "feedback/success.html")]
struct FeedbackSuccess<'a> {
    error: Option<ErrorPage<'a>>,
}

const PAGE: &str = "Feedback";

impl<'a> Default for FeedbackSuccess<'a> {
    fn default() -> Self {
        FeedbackSuccess { error: None }
    }
}

impl<'a> FeedbackSuccess<'a> {
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            error: Some(ErrorPage::new(title, message)),
        }
    }
}

lazy_static! {
    static ref INDEX: String = FeedbackSuccess::default().render_once().unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub redirect_to: Option<String>,
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.feedback.rating_form.strip_prefix(crate::V1_API_ROUTES.feedback.scope).unwrap()"
)]
pub async fn rating_form(
    rating_payload: web::Form<RatingReq>,
    state: web::Query<State>,
    path: web::Path<String>,
    data: AppData,
) -> PageResult<impl Responder> {
    let rating_payload = rating_payload.into_inner();
    let path = path.into_inner();
    let state = state.into_inner();
    let campaign_id = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    let (status, body) =
        match runners::rating(&campaign_id, &rating_payload, &data).await {
            Ok(_feedback_id) => (StatusCode::OK, None),
            Err(e) => {
                let status = e.status_code();
                let heading = status.canonical_reason().unwrap_or("Error");
                let page = FeedbackSuccess::new(heading, &format!("{}", e))
                    .render_once()
                    .unwrap();
                (status, Some(page))
            }
        };

    if state.redirect_to.is_some() {
        Ok(HttpResponseBuilder::new(status)
            .insert_header((header::LOCATION, state.redirect_to.unwrap()))
            .body(body.as_ref().unwrap_or(&*INDEX)))
    } else {
        Ok(HttpResponseBuilder::new(status).body(body.as_ref().unwrap_or(&*INDEX)))
    }
}
