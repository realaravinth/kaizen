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

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
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

pub mod routes {
    pub struct Campaign {
        pub new: &'static str,
        pub delete: &'static str,
        pub get: &'static str,
        pub get_all: &'static str,
    }

    impl Campaign {
        pub const fn new() -> Campaign {
            let new = "/api/v1/campaign/new";
            let delete = "/api/v1/campaign/{uuid}/delete";
            let get = "/api/v1/campaign/{uuid}delete";
            let get_all = "/api/v1/campaign/list";
            Campaign {
                new,
                delete,
                get,
                get_all,
            }
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(new);
    cfg.service(delete);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateReq {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateResp {
    pub uuid: String,
}

#[my_codegen::post(path = "crate::V1_API_ROUTES.campaign.new", wrap = "crate::CheckLogin")]
pub async fn new(
    id: Identity,
    payload: web::Json<CreateReq>,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let mut uuid;

    loop {
        uuid = get_uuid();

        let res = sqlx::query!(
            "INSERT INTO 
                kaizen_campaign (name , uuid, user_id) 
            VALUES 
                ($1, $2, 
                    (SELECT 
                        ID 
                    FROM 
                        kaizen_users 
                    WHERE 
                        name = $3
                    )
                )",
            &payload.name,
            &uuid,
            &username,
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

    let resp = CreateResp {
        uuid: uuid.to_string(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.campaign.delete",
    wrap = "crate::CheckLogin"
)]
pub async fn delete(
    id: Identity,
    data: AppData,
    path: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let uuid = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    sqlx::query!(
        "DELETE 
                    FROM kaizen_campaign 
                WHERE 
                    user_id = (SELECT 
                                    ID 
                                FROM 
                                    kaizen_users 
                                WHERE 
                                    name = $1
                            )
                AND
                    uuid = ($2)",
        &username,
        &uuid
    )
    .execute(&data.db)
    .await?;

    Ok(HttpResponse::Ok())
}
