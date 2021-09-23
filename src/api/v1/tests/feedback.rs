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
use crate::api::v1::feedback::{RatingReq, URL_MAX_LENGTH};
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

    delete_user(NAME, &data).await;
    let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);

    let uuid = create_new_campaign(CAMPAIGN_NAME, data.clone(), cookies.clone()).await;

    let mut rating = RatingReq {
        helpful: true,
        description: NAME.into(),
        page_url: url,
    };

    let add_feedback_route = add_feedback_route(&uuid);

    bad_post_req_test(
        NAME,
        PASSWORD,
        &add_feedback_route,
        &rating,
        ServiceError::URLTooLong,
    )
    .await;

    rating.page_url = rating.page_url[0..(rating.page_url.len() - 1)].into();

    add_feedback(&rating, &uuid, data.clone(), cookies.clone()).await;
}

#[actix_rt::test]
async fn feedback_works() {
    let data = Data::new().await;
    const NAME: &str = "testfeedbackuser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testfeedbackuser@a.com";
    const CAMPAIGN_NAME: &str = "testfeedbackuser";
    const PAGE_URL: &str = "http://example.com/foo";

    delete_user(NAME, &data).await;
    let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);

    let uuid = create_new_campaign(CAMPAIGN_NAME, data.clone(), cookies.clone()).await;

    let mut rating = RatingReq {
        helpful: true,
        description: NAME.into(),
        page_url: PAGE_URL.into(),
    };

    let bad_feedback_url = ROUTES.feedback.rating.replace("{campaign_id}", NAME);

    bad_post_req_test(
        NAME,
        PASSWORD,
        &bad_feedback_url,
        &rating,
        ServiceError::NotAnId,
    )
    .await;

    rating.page_url = "foo".into();
    bad_post_req_test(
        NAME,
        PASSWORD,
        &bad_feedback_url,
        &rating,
        ServiceError::NotAUrl,
    )
    .await;

    rating.page_url = get_random(URL_MAX_LENGTH + 1);
    bad_post_req_test(
        NAME,
        PASSWORD,
        &bad_feedback_url,
        &rating,
        ServiceError::URLTooLong,
    )
    .await;

    rating.page_url = PAGE_URL.into();

    add_feedback(&rating, &uuid, data.clone(), cookies.clone()).await;

    let feedbacks = get_feedback(&uuid, data, cookies).await;
    assert!(feedbacks.iter().any(|f| f.description == NAME));
}

#[actix_rt::test]
async fn feedback_duplicate_page_url_works() {
    let data = Data::new().await;
    const NAME: &str = "testfeedbackspamuser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testfeedbackusespamr@a.com";
    const CAMPAIGN_NAME: &str = "testfeedbacspamkuser";
    const PAGE_URL: &str = "http://spam.example.com/foo";

    delete_user(NAME, &data).await;
    let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);

    let uuid = create_new_campaign(CAMPAIGN_NAME, data.clone(), cookies.clone()).await;

    let rating = RatingReq {
        helpful: true,
        description: NAME.into(),
        page_url: PAGE_URL.into(),
    };

    let count = 5;
    let mut feedback_ids = Vec::with_capacity(count);
    for _ in 0..count {
        let feedback_id =
            add_feedback(&rating, &uuid, data.clone(), cookies.clone()).await;
        feedback_ids.push(feedback_id);
    }

    let feedbacks = get_feedback(&uuid, data, cookies).await;

    assert_eq!(feedbacks.len(), count);
    feedbacks
        .iter()
        .for_each(|f| println!("{:?}", f.description));
    assert!(feedbacks.iter().any(|f| f.description == NAME));
}
