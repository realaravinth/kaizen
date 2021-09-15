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
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

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
    pub struct Feedback {
        pub rating: &'static str,
        pub description: &'static str,
    }

    impl Feedback {
        pub const fn new() -> Feedback {
            let rating = "/api/v1/feedback/rating";
            let description = "/api/v1/feedback/description";
            Feedback {
                rating,
                description,
            }
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    //   cfg.service(rating);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RatingReq {
    pub helpful: bool,
    pub description: Option<String>,
}

//#[my_codegen::post(path = "crate::V1_API_ROUTES.feedback.rating")]
//pub async fn rating(payload: web::Json<RatingReq>, data: AppData) -> ServiceResult<impl Responder> {
//    unimplemented!()
//    //    Ok(HttpResponse::Ok().json(resp))
//}
