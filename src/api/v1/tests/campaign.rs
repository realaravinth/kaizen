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

use crate::api::v1::campaign::{CreateReq, CreateResp, ListCampaignResp};
use crate::api::v1::ROUTES;
use crate::data::Data;
use crate::errors::*;
use crate::*;

use crate::tests::*;

#[actix_rt::test]
async fn campaign_works() {
    let data = Data::new().await;
    const NAME: &str = "testcampaignuser";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testcampaignuser@a.com";
    const CAMPAIGN_NAME: &str = "testcampaignuser";

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

    let list_resp = test::call_service(
        &app,
        post_request!(ROUTES.campaign.list)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list: Vec<ListCampaignResp> = test::read_body_json(list_resp).await;
    assert!(list.iter().any(|c| c.name == CAMPAIGN_NAME));

    bad_post_req_test_witout_payload(
        NAME,
        PASSWORD,
        &ROUTES.campaign.delete.replace("{uuid}", NAME),
        ServiceError::NotAnId,
    )
    .await;

    let del_route = ROUTES.campaign.delete.replace("{uuid}", &uuid.uuid);
    println!("del route: {}", &del_route);
    let del_resp = test::call_service(
        &app,
        post_request!(&del_route)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(del_resp.status(), StatusCode::OK);

    let list_resp_post_deletion = test::call_service(
        &app,
        post_request!(ROUTES.campaign.list)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(list_resp_post_deletion.status(), StatusCode::OK);
    let list: Vec<ListCampaignResp> =
        test::read_body_json(list_resp_post_deletion).await;
    assert!(!list.iter().any(|c| c.name == CAMPAIGN_NAME));
}
