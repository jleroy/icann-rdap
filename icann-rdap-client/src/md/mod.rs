//! Converts RDAP to Markdown.

use {
    crate::rdap::rr::RequestData,
    buildstructor::Builder,
    icann_rdap_common::{check::CheckParams, httpdata::HttpData, response::RdapResponse},
    std::{any::TypeId, char},
    strum::EnumMessage,
};

use icann_rdap_common::check::{CheckClass, Checks, CHECK_CLASS_LEN};

use self::string::StringUtil;

pub mod autnum;
pub mod domain;
pub mod entity;
pub mod error;
pub mod help;
pub mod nameserver;
pub mod network;
pub mod redacted;
pub mod search;
pub mod string;
pub mod table;
pub mod types;

pub(crate) const _CODE_INDENT: &str = "    ";

pub(crate) const HR: &str = "----------------------------------------\n";

/// Specifies options for generating markdown.
pub struct MdOptions {
    /// If true, do not use Unicode characters.
    pub no_unicode_chars: bool,

    /// The character used for text styling of bold and italics.
    pub text_style_char: char,

    /// If true, headers use the hash marks or under lines.
    pub hash_headers: bool,

    /// If true, the text_style_char will appear in a justified text.
    pub style_in_justify: bool,
}

impl Default for MdOptions {
    fn default() -> Self {
        Self {
            no_unicode_chars: false,
            text_style_char: '*',
            hash_headers: true,
            style_in_justify: false,
        }
    }
}

impl MdOptions {
    /// Defaults for markdown that looks more like plain text.
    pub fn plain_text() -> Self {
        Self {
            no_unicode_chars: true,
            text_style_char: '_',
            hash_headers: false,
            style_in_justify: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MdParams<'a> {
    pub heading_level: usize,
    pub root: &'a RdapResponse,
    pub http_data: &'a HttpData,
    pub parent_type: TypeId,
    pub check_types: &'a [CheckClass],
    pub options: &'a MdOptions,
    pub req_data: &'a RequestData<'a>,
}

impl MdParams<'_> {
    pub fn from_parent(&self, parent_type: TypeId) -> Self {
        Self {
            parent_type,
            heading_level: self.heading_level,
            root: self.root,
            http_data: self.http_data,
            check_types: self.check_types,
            options: self.options,
            req_data: self.req_data,
        }
    }

    pub fn next_level(&self) -> Self {
        Self {
            heading_level: self.heading_level + 1,
            ..*self
        }
    }
}

pub trait ToMd {
    fn to_md(&self, params: MdParams) -> String;
}

impl ToMd for RdapResponse {
    fn to_md(&self, params: MdParams) -> String {
        let mut md = String::new();
        md.push_str(&params.http_data.to_md(params));
        let variant_md = match &self {
            Self::Entity(entity) => entity.to_md(params),
            Self::Domain(domain) => domain.to_md(params),
            Self::Nameserver(nameserver) => nameserver.to_md(params),
            Self::Autnum(autnum) => autnum.to_md(params),
            Self::Network(network) => network.to_md(params),
            Self::DomainSearchResults(results) => results.to_md(params),
            Self::EntitySearchResults(results) => results.to_md(params),
            Self::NameserverSearchResults(results) => results.to_md(params),
            Self::ErrorResponse(error) => error.to_md(params),
            Self::Help(help) => help.to_md(params),
        };
        md.push_str(&variant_md);
        md
    }
}

pub trait MdUtil {
    fn get_header_text(&self) -> MdHeaderText;
}

#[derive(Builder)]
pub struct MdHeaderText {
    header_text: String,
    children: Vec<MdHeaderText>,
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for MdHeaderText {
    fn to_string(&self) -> String {
        self.header_text.clone()
    }
}

impl MdUtil for RdapResponse {
    fn get_header_text(&self) -> MdHeaderText {
        match &self {
            Self::Entity(entity) => entity.get_header_text(),
            Self::Domain(domain) => domain.get_header_text(),
            Self::Nameserver(nameserver) => nameserver.get_header_text(),
            Self::Autnum(autnum) => autnum.get_header_text(),
            Self::Network(network) => network.get_header_text(),
            Self::DomainSearchResults(results) => results.get_header_text(),
            Self::EntitySearchResults(results) => results.get_header_text(),
            Self::NameserverSearchResults(results) => results.get_header_text(),
            Self::ErrorResponse(error) => error.get_header_text(),
            Self::Help(help) => help.get_header_text(),
        }
    }
}

pub(crate) fn checks_ul(checks: &Checks, params: MdParams) -> String {
    let mut md = String::new();
    checks
        .items
        .iter()
        .filter(|item| params.check_types.contains(&item.check_class))
        .for_each(|item| {
            md.push_str(&format!(
                "* {}: {}\n",
                &item
                    .check_class
                    .to_string()
                    .to_right_em(*CHECK_CLASS_LEN, params.options),
                item.check
                    .get_message()
                    .expect("Check has no message. Coding error.")
            ))
        });
    md
}

pub(crate) trait FromMd<'a> {
    fn from_md(md_params: MdParams<'a>, parent_type: TypeId) -> Self;
    fn from_md_no_parent(md_params: MdParams<'a>) -> Self;
}

impl<'a> FromMd<'a> for CheckParams<'a> {
    fn from_md(md_params: MdParams<'a>, parent_type: TypeId) -> Self {
        Self {
            do_subchecks: false,
            root: md_params.root,
            parent_type,
            allow_unreg_ext: false,
        }
    }

    fn from_md_no_parent(md_params: MdParams<'a>) -> Self {
        Self {
            do_subchecks: false,
            root: md_params.root,
            parent_type: md_params.parent_type,
            allow_unreg_ext: false,
        }
    }
}
