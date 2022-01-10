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

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder};
use futures::try_join;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

use super::get_uuid;
use crate::errors::*;
use crate::AppData;

pub mod routes {
    pub struct Campaign {
        pub new: &'static str,
        pub delete: &'static str,
        pub get_feedback: &'static str,
        pub list: &'static str,
    }

    impl Campaign {
        pub const fn new() -> Campaign {
            let new = "/api/v1/campaign/new";
            let delete = "/api/v1/campaign/{uuid}/delete";
            let get_feedback = "/api/v1/campaign/{uuid}/feedback";
            let list = "/api/v1/campaign/list";
            Campaign {
                new,
                delete,
                get_feedback,
                list,
            }
        }

        pub fn get_feedback_route(&self, campaign_id: &str) -> String {
            self.get_feedback.replace("{uuid}", &campaign_id)
        }

        pub fn get_delete_route(&self, campaign_id: &str) -> String {
            self.delete.replace("{uuid}", &campaign_id)
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(new);
    cfg.service(delete);
    cfg.service(list_campaign);
    cfg.service(get_feedback);
}

pub mod runners {
    use super::*;

    pub async fn new(
        payload: &CreateReq,
        data: &AppData,
        id: &Identity,
    ) -> ServiceResult<CreateResp> {
        let mut uuid;
        let username = id.identity().unwrap();

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

        Ok(CreateResp {
            uuid: uuid.to_string(),
        })
    }

    pub async fn list_campaign_runner(
        username: &str,
        data: &AppData,
    ) -> ServiceResult<Vec<ListCampaignResp>> {
        struct ListCampaign {
            name: String,
            uuid: Uuid,
        }

        let mut campaigns = sqlx::query_as!(
            ListCampaign,
            "SELECT 
            name, uuid
        FROM 
            kaizen_campaign 
            WHERE
                user_id = (
                    SELECT 
                        ID
                    FROM 
                        kaizen_users
                    WHERE
                        name = $1
                )",
            username
        )
        .fetch_all(&data.db)
        .await?;

        let mut list_resp = Vec::with_capacity(campaigns.len());
        campaigns.drain(0..).for_each(|c| {
            list_resp.push(ListCampaignResp {
                name: c.name,
                uuid: c.uuid.to_string(),
            });
        });

        Ok(list_resp)
    }

    pub async fn get_feedback(
        username: &str,
        uuid: &str,
        data: &AppData,
    ) -> ServiceResult<GetFeedbackResp> {
        let uuid = Uuid::parse_str(uuid).map_err(|_| ServiceError::NotAnId)?;

        struct FeedbackInternal {
            time: OffsetDateTime,
            description: String,
            helpful: bool,
        }

        struct Name {
            name: String,
        }

        let name_fut = sqlx::query_as!(
            Name,
            "SELECT name 
            FROM kaizen_campaign 
            WHERE uuid = $1 
            AND
                user_id = (
                    SELECT 
                        ID
                    FROM 
                        kaizen_users
                    WHERE
                        name = $2
                )
           ",
            uuid,
            username
        )
        .fetch_one(&data.db); //.await?;

        let feedback_fut = sqlx::query_as!(
            FeedbackInternal,
            "SELECT 
            time, description, helpful
        FROM 
            kaizen_feedbacks
        WHERE campaign_id = (
            SELECT uuid 
            FROM 
                kaizen_campaign
            WHERE
                uuid = $1
            AND
                user_id = (
                    SELECT 
                        ID
                    FROM 
                        kaizen_users
                    WHERE
                        name = $2
                )
           )",
            uuid,
            username
        )
        .fetch_all(&data.db);
        let (name, mut feedbacks) = try_join!(name_fut, feedback_fut)?;
        //.await?;

        let mut feedback_resp = Vec::with_capacity(feedbacks.len());
        feedbacks.drain(0..).for_each(|f| {
            feedback_resp.push(Feedback {
                time: f.time.unix_timestamp() as u64,
                description: f.description,
                helpful: f.helpful,
            });
        });

        Ok(GetFeedbackResp {
            feedbacks: feedback_resp,
            name: name.name,
        })
    }

    pub async fn delete(
        uuid: &Uuid,
        username: &str,
        data: &AppData,
    ) -> ServiceResult<()> {
        sqlx::query!(
            "DELETE 
            FROM kaizen_campaign 
         WHERE 
             user_id = (
                 SELECT 
                         ID 
                 FROM 
                         kaizen_users 
                 WHERE 
                         name = $1
             )
         AND
            uuid = ($2)",
            username,
            uuid
        )
        .execute(&data.db)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateReq {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateResp {
    pub uuid: String,
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.campaign.new",
    wrap = "crate::get_auth_middleware()"
)]
pub async fn new(
    id: Identity,
    payload: web::Json<CreateReq>,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let resp = runners::new(&payload.into_inner(), &data, &id).await?;
    Ok(HttpResponse::Ok().json(resp))
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.campaign.delete",
    wrap = "crate::get_auth_middleware()"
)]
pub async fn delete(
    id: Identity,
    data: AppData,
    path: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let uuid = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;
    runners::delete(&uuid, &username, &data).await?;
    Ok(HttpResponse::Ok())
}

#[derive(Serialize, Deserialize)]
pub struct Feedback {
    pub time: u64,
    pub description: String,
    pub helpful: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetFeedbackResp {
    pub name: String,
    pub feedbacks: Vec<Feedback>,
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.campaign.get_feedback",
    wrap = "crate::get_auth_middleware()"
)]
pub async fn get_feedback(
    id: Identity,
    data: AppData,
    path: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let feedback_resp = runners::get_feedback(&username, &path, &data).await?;
    Ok(HttpResponse::Ok().json(feedback_resp))
}

#[derive(Serialize, Deserialize)]
pub struct ListCampaignResp {
    pub name: String,
    pub uuid: String,
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.campaign.list",
    wrap = "crate::get_auth_middleware()"
)]
pub async fn list_campaign(
    id: Identity,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let list_resp = runners::list_campaign_runner(&username, &data).await?;

    Ok(HttpResponse::Ok().json(list_resp))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::api::v1::ROUTES;
    use crate::data::Data;
    use crate::*;

    use crate::tests::*;

    #[actix_rt::test]
    async fn campaign_works() {
        let data = Data::new().await;
        const NAME: &str = "testcampaignuser";
        const PASSWORD: &str = "longpassword";
        const EMAIL: &str = "testcampaignuser@a.com";
        const CAMPAIGN_NAME: &str = "testcampaignuser";

        delete_user(NAME, &data).await;
        let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);

        let uuid = create_new_campaign(NAME, data.clone(), cookies.clone()).await;

        let list = list_campaings(data.clone(), cookies.clone()).await;
        assert!(list.iter().any(|c| c.name == CAMPAIGN_NAME));

        bad_post_req_test_witout_payload(
            NAME,
            PASSWORD,
            &ROUTES.campaign.delete.replace("{uuid}", NAME),
            ServiceError::NotAnId,
        )
        .await;

        delete_campaign(&uuid, data.clone(), cookies.clone()).await;

        let list = list_campaings(data.clone(), cookies.clone()).await;
        assert!(!list.iter().any(|c| c.name == CAMPAIGN_NAME));
    }
}
