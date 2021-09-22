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
use std::convert::From;

use actix_identity::Identity;
use actix_web::HttpResponseBuilder;
use actix_web::{error::ResponseError, http::header, web, HttpResponse, Responder};
use lazy_static::lazy_static;
use my_codegen::{get, post};
use sailfish::TemplateOnce;

use crate::api::v1::campaign::{runners, CreateReq, GetFeedbackResp};
use crate::errors::*;
use crate::pages::errors::ErrorPage;
use crate::AppData;
use crate::PAGES;

#[derive(TemplateOnce)]
#[template(path = "panel/campaigns/get/index.html")]
struct ViewFeedback<'a> {
    error: Option<ErrorPage<'a>>,
    feedbacks: Vec<GetFeedbackResp>,
    uuid: &'a str,
}

const PAGE: &str = "New Campaign";

impl<'a> ViewFeedback<'a> {
    pub fn set_error(&mut self, title: &'a str, message: &'a str) {
        self.error = Some(ErrorPage::new(title, message));
    }

    pub fn new(feedbacks: Vec<GetFeedbackResp>, uuid: &'a str) -> Self {
        Self {
            feedbacks,
            uuid,
            error: None,
        }
    }
}

#[get(
    path = "PAGES.panel.campaigns.get_feedback",
    wrap = "crate::CheckLogin"
)]
pub async fn get_feedback(
    id: Identity,
    data: AppData,
    path: web::Path<String>,
) -> impl Responder {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let feedback_resp = runners::get_feedback(&username, &path, &data)
        .await
        .unwrap();
    let page = ViewFeedback::new(feedback_resp, &path)
        .render_once()
        .unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}

//#[post(path = "PAGES.panel.campaigns.new", wrap = "crate::CheckLogin")]
//pub async fn new_campaign_submit(
//    id: Identity,
//    payload: web::Form<CreateReq>,
//    data: AppData,
//) -> PageResult<impl Responder> {
//    match runners::new(&payload.into_inner(), &data, &id).await {
//        Ok(_) => {
//            Ok(HttpResponse::Found()
//                //TODO show stats of new campaign
//                .insert_header((header::LOCATION, PAGES.panel.campaigns.home))
//                .finish())
//        }
//        Err(e) => {
//            let status = e.status_code();
//            let heading = status.canonical_reason().unwrap_or("Error");
//
//            Ok(HttpResponseBuilder::new(status)
//                .content_type("text/html; charset=utf-8")
//                .body(
//                    ViewFeedback::new(heading, &format!("{}", e))
//                        .render_once()
//                        .unwrap(),
//                ))
//        }
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use actix_web::test;
//
//    use super::*;
//
//    use crate::data::Data;
//    use crate::tests::*;
//    use crate::*;
//    use actix_web::http::StatusCode;
//
//    #[actix_rt::test]
//    async fn new_campaign_form_works() {
//        let data = Data::new().await;
//        const NAME: &str = "testusercampaignform";
//        const EMAIL: &str = "testcampaignuser@aaa.com";
//        const PASSWORD: &str = "longpassword";
//
//        const CAMPAIGN_NAME: &str = "testcampaignuser";
//
//        let app = get_app!(data).await;
//        delete_user(NAME, &data).await;
//        let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
//        let cookies = get_cookie!(signin_resp);
//
//        let new = CreateReq {
//            name: CAMPAIGN_NAME.into(),
//        };
//
//        let new_resp = test::call_service(
//            &app,
//            post_request!(&new, PAGES.panel.campaigns.new, FORM)
//                .cookie(cookies.clone())
//                .to_request(),
//        )
//        .await;
//        assert_eq!(new_resp.status(), StatusCode::FOUND);
//        let headers = new_resp.headers();
//        assert_eq!(
//            headers.get(header::LOCATION).unwrap(),
//            PAGES.panel.campaigns.home,
//        );
//    }
//}
