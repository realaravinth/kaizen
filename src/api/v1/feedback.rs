/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
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

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use super::get_uuid;
use crate::errors::*;
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
        pub description: &'static str,
    }

    impl Feedback {
        pub const fn new() -> Feedback {
            let rating = "/api/v1/feedback/{campaign_id}/rating";
            let description = "/api/v1/feedback/{feedback_id}/description";
            Feedback {
                rating,
                description,
            }
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(rating);
    cfg.service(description);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RatingReq {
    pub helpful: bool,
    pub description: Option<String>,
    pub page_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RatingResp {
    pub uuid: String,
}

#[my_codegen::post(path = "crate::V1_API_ROUTES.feedback.rating")]
pub async fn rating(
    payload: web::Json<RatingReq>,
    path: web::Path<String>,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let payload = payload.into_inner();
    if payload.page_url.len() > URL_MAX_LENGTH {
        return Err(ServiceError::URLTooLong);
    }
    Url::parse(&payload.page_url)?;

    let path = path.into_inner();
    let campaign_id = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    let now = OffsetDateTime::now_utc();

    let mut uuid;

    loop {
        uuid = get_uuid();

        let res = if payload.description.is_some() {
            sqlx::query!(
                "INSERT INTO 
                kaizen_feedback (helpful , description, uuid, campaign_id, time) 
            VALUES 
                ($1, $2, $3, $4, $5)",
                &payload.helpful,
                &payload.description.as_ref().unwrap(),
                &uuid,
                &campaign_id,
                &now,
            )
            .execute(&data.db)
            .await
        } else {
            sqlx::query!(
                "INSERT INTO 
                kaizen_feedback (helpful, uuid, campaign_id, time) 
            VALUES 
                ($1, $2, $3, $4)",
                &payload.helpful,
                &uuid,
                &campaign_id,
                &now,
            )
            .execute(&data.db)
            .await
        };

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

    Ok(HttpResponse::Ok().json(resp))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DescriptionReq {
    pub description: String,
}

#[my_codegen::post(path = "crate::V1_API_ROUTES.feedback.description")]
pub async fn description(
    payload: web::Json<DescriptionReq>,
    path: web::Path<String>,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let path = path.into_inner();
    let payload = payload.into_inner();
    let feedback_id = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    let now = OffsetDateTime::now_utc();

    sqlx::query!(
        "UPDATE kaizen_feedback 
        SET 
            description = $1, time  = $2
        WHERE uuid = $3",
        &payload.description,
        &now,
        &feedback_id,
    )
    .execute(&data.db)
    .await?;

    Ok(HttpResponse::Ok())
}
