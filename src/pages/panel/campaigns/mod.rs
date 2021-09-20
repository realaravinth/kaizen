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
 * You should have received a copy of the GNU Affero General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder};
use my_codegen::get;
use sailfish::TemplateOnce;

use crate::api::v1::campaign::{runner::list_campaign_runner, ListCampaignResp};
use crate::AppData;
use crate::PAGES;

pub mod routes {
    pub struct Campaigns {
        pub home: &'static str,
    }
    impl Campaigns {
        pub const fn new() -> Campaigns {
            Campaigns { home: "/campaigns" }
        }

        pub const fn get_sitemap() -> [&'static str; 1] {
            const CAMPAIGNS: Campaigns = Campaigns::new();
            [CAMPAIGNS.home]
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(home);
}

#[derive(TemplateOnce)]
#[template(path = "panel/campaigns/index.html")]
struct HomePage {
    data: Vec<ListCampaignResp>,
}

impl HomePage {
    fn new(data: Vec<ListCampaignResp>) -> Self {
        Self { data }
    }
}

const PAGE: &str = "Campaigns";

#[get(path = "PAGES.panel.campaigns.home", wrap = "crate::CheckLogin")]
pub async fn home(data: AppData, id: Identity) -> impl Responder {
    let username = id.identity().unwrap();
    let campaigns = list_campaign_runner(&username, &data).await.unwrap();
    let page = HomePage::new(campaigns).render_once().unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(&page)
}
