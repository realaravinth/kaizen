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
use crate::api::v1::feedback::{RatingReq, RatingResp, URL_MAX_LENGTH};
use crate::api::v1::{get_random, ROUTES};
use crate::data::Data;
use crate::errors::*;
use crate::*;

use crate::tests::*;

#[actix_rt::test]
async fn feedback_page_url_length() {
    let data = Data::new().await;
    const NAME: &str = "testfeedbackurluser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testfeedbackurluser@a.com";
    const CAMPAIGN_NAME: &str = "testfeedbackurluser";
    const PAGE_URL: &str = "http://example.com/foo";
    let url = format!(
        "{}/{}",
        PAGE_URL,
        get_random(URL_MAX_LENGTH - PAGE_URL.len())
    );

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

    let mut rating = RatingReq {
        helpful: true,
        description: NAME.into(),
        page_url: url,
    };

    let add_feedback_route = ROUTES.feedback.rating.replace("{campaign_id}", &uuid.uuid);

    bad_post_req_test(
        NAME,
        PASSWORD,
        &add_feedback_route,
        &rating,
        ServiceError::URLTooLong,
    )
    .await;

    rating.page_url = rating.page_url[0..(rating.page_url.len() - 1)].into();

    //    rating.page_url = PAGE_URL.into();
    let add_feedback_resp = test::call_service(
        &app,
        post_request!(&rating, &add_feedback_route)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(add_feedback_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn feedback_works() {
    let data = Data::new().await;
    const NAME: &str = "testfeedbackuser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testfeedbackuser@a.com";
    const CAMPAIGN_NAME: &str = "testfeedbackuser";
    const PAGE_URL: &str = "http://example.com/foo";

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

    let mut rating = RatingReq {
        helpful: true,
        description: NAME.into(),
        page_url: PAGE_URL.into(),
    };

    bad_post_req_test(
        NAME,
        PASSWORD,
        &ROUTES.feedback.rating.replace("{campaign_id}", NAME),
        &rating,
        ServiceError::NotAnId,
    )
    .await;

    rating.page_url = "foo".into();
    bad_post_req_test(
        NAME,
        PASSWORD,
        &ROUTES.feedback.rating.replace("{campaign_id}", NAME),
        &rating,
        ServiceError::NotAUrl,
    )
    .await;

    rating.page_url = get_random(URL_MAX_LENGTH + 1);
    bad_post_req_test(
        NAME,
        PASSWORD,
        &ROUTES.feedback.rating.replace("{campaign_id}", NAME),
        &rating,
        ServiceError::URLTooLong,
    )
    .await;

    rating.page_url = PAGE_URL.into();
    let add_feedback_route = ROUTES.feedback.rating.replace("{campaign_id}", &uuid.uuid);
    let add_feedback_resp = test::call_service(
        &app,
        post_request!(&rating, &add_feedback_route)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(add_feedback_resp.status(), StatusCode::OK);

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
    assert!(feedback.iter().any(|f| f.description == NAME));
}

#[actix_rt::test]
async fn feedback_duplicate_page_url_works() {
    let data = Data::new().await;
    const NAME: &str = "testfeedbackspamuser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testfeedbackusespamr@a.com";
    const CAMPAIGN_NAME: &str = "testfeedbacspamkuser";
    const PAGE_URL: &str = "http://spam.example.com/foo";

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
        description: NAME.into(),
        page_url: PAGE_URL.into(),
    };

    let add_feedback_route = ROUTES.feedback.rating.replace("{campaign_id}", &uuid.uuid);
    let count = 5;
    let mut feedback_ids = Vec::with_capacity(count);
    for _ in 0..count {
        let add_feedback_resp = test::call_service(
            &app,
            post_request!(&rating, &add_feedback_route)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        let status = add_feedback_resp.status();
        if status != StatusCode::OK {
            let resp_err: ErrorToResponse =
                test::read_body_json(add_feedback_resp).await;
            println!("{}", resp_err.error);
            panic!();
        }
        //assert_eq!(add_feedback_resp.status(), StatusCode::OK);

        let feedback_id: RatingResp = test::read_body_json(add_feedback_resp).await;
        feedback_ids.push(feedback_id);
    }

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
    assert_eq!(feedback.len(), count);
    feedback
        .iter()
        .for_each(|f| println!("{:?}", f.description));
    assert!(feedback.iter().any(|f| f.description == NAME));
}
