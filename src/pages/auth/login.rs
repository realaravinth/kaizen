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

use actix_web::{HttpResponse, Responder};
use lazy_static::lazy_static;
use my_codegen::get;
use sailfish::TemplateOnce;

use crate::PAGES;

#[derive(Clone, TemplateOnce)]
#[template(path = "auth/login/index.html")]
struct IndexPage;

const PAGE: &str = "Login";

impl Default for IndexPage {
    fn default() -> Self {
        IndexPage
    }
}

lazy_static! {
    static ref INDEX: String = IndexPage::default().render_once().unwrap();
}

#[get(path = "PAGES.auth.login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(&*INDEX)
}
