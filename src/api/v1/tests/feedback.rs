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
use actix_web::http::StatusCode;
use actix_web::test;

use crate::api::v1::campaign::{CreateReq, CreateResp, GetFeedbackResp};
use crate::api::v1::feedback::{DescriptionReq, RatingReq, RatingResp};
use crate::api::v1::ROUTES;
use crate::data::Data;
use crate::errors::*;
use crate::*;

use crate::tests::*;

#[actix_rt::test]
async fn feedback_works() {
    let data = Data::new().await;
    const NAME: &str = "testfeedbackuser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testfeedbackuser@a.com";
    const CAMPAIGN_NAME: &str = "testfeedbackuser";

    let app = get_app!(data).await;
    delete_user(NAME, &data).await;
    let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);

    let new = CreateReq {
        name: CAMPAIGN_NAME.into(),
    };

    let new_resp = test::call_service(
        &app,
        post_request!(&new, ROUTES.campaign.new)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(new_resp.status(), StatusCode::OK);
    let uuid: CreateResp = test::read_body_json(new_resp).await;

    let rating = RatingReq {
        helpful: true,
        description: None,
    };
    bad_post_req_test(
        NAME,
        PASSWORD,
        &ROUTES.feedback.rating.replace("{campaign_id}", NAME),
        &rating,
        ServiceError::NotAnId,
    )
    .await;

    let add_feedback_route = ROUTES.feedback.rating.replace("{campaign_id}", &uuid.uuid);
    let add_feedback_resp = test::call_service(
        &app,
        post_request!(&rating, &add_feedback_route)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(add_feedback_resp.status(), StatusCode::OK);
    let feedback_id: RatingResp = test::read_body_json(add_feedback_resp).await;

    let description_req = DescriptionReq {
        description: NAME.into(),
    };

    bad_post_req_test(
        NAME,
        PASSWORD,
        &ROUTES.feedback.description.replace("{feedback_id}", NAME),
        &description_req,
        ServiceError::NotAnId,
    )
    .await;

    let add_description_route = ROUTES
        .feedback
        .description
        .replace("{feedback_id}", &feedback_id.uuid);
    let set_description_resp = test::call_service(
        &app,
        post_request!(&description_req, &add_description_route).to_request(),
    )
    .await;
    assert_eq!(set_description_resp.status(), StatusCode::OK);

    bad_post_req_test(
        NAME,
        PASSWORD,
        &ROUTES.campaign.get_feedback.replace("{uuid}", NAME),
        &rating,
        ServiceError::NotAnId,
    )
    .await;

    let get_feedback_route = ROUTES.campaign.get_feedback.replace("{uuid}", &uuid.uuid);
    let get_feedback_resp = test::call_service(
        &app,
        post_request!(&get_feedback_route)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(get_feedback_resp.status(), StatusCode::OK);
    let feedback: Vec<GetFeedbackResp> = test::read_body_json(get_feedback_resp).await;
    feedback.iter().any(|f| f.description == Some(NAME.into()));
}
