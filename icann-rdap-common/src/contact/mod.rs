//! Easy representation of contact information found in an Entity.
//!
//! This module converts contact information to and from vCard/jCard, which is hard to
//! work with directly. It is also intended as a way of bridging the between vCard/jCard
//! and any new contact model.
//!
//! This struct can be built using the builder.
//!
//! ```rust
//! use icann_rdap_common::contact::Contact;
//!
//! let contact = Contact::builder()
//!   .kind("individual")
//!   .full_name("Bob Smurd")
//!   .build();
//! ```
//!
//! Once built, a Contact struct can be converted to an array of [serde_json::Value]'s,
//! which can be used with serde to serialize to JSON.
//!
//! ```rust
//! use icann_rdap_common::contact::Contact;
//! use serde::Serialize;
//! use serde_json::Value;
//!
//! let contact = Contact::builder()
//!   .kind("individual")
//!   .full_name("Bob Smurd")
//!   .build();
//!
//! let v = contact.to_vcard();
//! let json = serde_json::to_string(&v);
//! ```
//!
//! To deserialize, use the `from_vcard` function.
//!
//! ```rust
//! use icann_rdap_common::contact::Contact;
//! use serde::Deserialize;
//! use serde_json::Value;
//!
//! let json = r#"
//! [
//!   "vcard",
//!   [
//!     ["version", {}, "text", "4.0"],
//!     ["fn", {}, "text", "Joe User"],
//!     ["kind", {}, "text", "individual"],
//!     ["org", {
//!       "type":"work"
//!     }, "text", "Example"],
//!     ["title", {}, "text", "Research Scientist"],
//!     ["role", {}, "text", "Project Lead"],
//!     ["adr",
//!       { "type":"work" },
//!       "text",
//!       [
//!         "",
//!         "Suite 1234",
//!         "4321 Rue Somewhere",
//!         "Quebec",
//!         "QC",
//!         "G1V 2M2",
//!         "Canada"
//!       ]
//!     ],
//!     ["tel",
//!       { "type":["work", "voice"], "pref":"1" },
//!       "uri", "tel:+1-555-555-1234;ext=102"
//!     ],
//!     ["email",
//!       { "type":"work" },
//!       "text", "joe.user@example.com"
//!     ]
//!   ]
//! ]"#;
//!
//! let data: Vec<Value> = serde_json::from_str(json).unwrap();
//! let contact = Contact::from_vcard(&data);
//! ```

mod from_vcard;
mod to_vcard;

use std::fmt::Display;

use buildstructor::Builder;

use crate::prelude::to_opt_vec;

/// Represents a contact. This more closely represents an EPP Contact with some
/// things taken from JSContact.
///
/// Using the builder to create the Contact:
/// ```rust
/// use icann_rdap_common::contact::Contact;
///
/// let contact = Contact::builder()
///   .kind("individual")
///   .full_name("Bob Smurd")
///   .build();
/// ```
///
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Contact {
    /// Preferred languages.
    pub langs: Option<Vec<Lang>>,

    /// The kind such as individual, company, etc...
    pub kind: Option<String>,

    /// Full name of the contact.
    pub full_name: Option<String>,

    /// Structured parts of the name.
    pub name_parts: Option<NameParts>,

    /// Nick names.
    pub nick_names: Option<Vec<String>>,

    /// Titles.
    pub titles: Option<Vec<String>>,

    /// Organizational Roles
    pub roles: Option<Vec<String>>,

    /// Organization names.
    pub organization_names: Option<Vec<String>>,

    /// Postal addresses.
    pub postal_addresses: Option<Vec<PostalAddress>>,

    /// Email addresses.
    pub emails: Option<Vec<Email>>,

    /// Phone numbers.
    pub phones: Option<Vec<Phone>>,

    /// Contact URIs.
    pub contact_uris: Option<Vec<String>>,

    /// URLs
    pub urls: Option<Vec<String>>,
}

#[buildstructor::buildstructor]
impl Contact {
    #[builder(visibility = "pub")]
    fn new(
        langs: Vec<Lang>,
        kind: Option<String>,
        full_name: Option<String>,
        name_parts: Option<NameParts>,
        nick_names: Vec<String>,
        titles: Vec<String>,
        roles: Vec<String>,
        organization_names: Vec<String>,
        postal_addresses: Vec<PostalAddress>,
        emails: Vec<Email>,
        phones: Vec<Phone>,
        contact_uris: Vec<String>,
        urls: Vec<String>,
    ) -> Self {
        Self {
            langs: to_opt_vec(langs),
            kind,
            full_name,
            name_parts,
            nick_names: to_opt_vec(nick_names),
            titles: to_opt_vec(titles),
            roles: to_opt_vec(roles),
            organization_names: to_opt_vec(organization_names),
            postal_addresses: to_opt_vec(postal_addresses),
            emails: to_opt_vec(emails),
            phones: to_opt_vec(phones),
            contact_uris: to_opt_vec(contact_uris),
            urls: to_opt_vec(urls),
        }
    }

    /// Returns false if there is data in the Contact.
    pub fn is_non_empty(&self) -> bool {
        self.langs.is_some()
            || self.kind.is_some()
            || self.full_name.is_some()
            || self.name_parts.is_some()
            || self.nick_names.is_some()
            || self.titles.is_some()
            || self.roles.is_some()
            || self.organization_names.is_some()
            || self.postal_addresses.is_some()
            || self.emails.is_some()
            || self.phones.is_some()
            || self.contact_uris.is_some()
            || self.urls.is_some()
    }

    /// Set the set of emails.
    pub fn set_emails(mut self, emails: &[impl ToString]) -> Self {
        let emails: Vec<Email> = emails
            .iter()
            .map(|e| Email::builder().email(e.to_string()).build())
            .collect();
        self.emails = (!emails.is_empty()).then_some(emails);
        self
    }

    /// Add a voice phone to the set of phones.
    pub fn add_voice_phones(mut self, phones: &[impl ToString]) -> Self {
        let mut phones: Vec<Phone> = phones
            .iter()
            .map(|p| {
                Phone::builder()
                    .contexts(vec!["voice".to_string()])
                    .phone(p.to_string())
                    .build()
            })
            .collect();
        if let Some(mut self_phones) = self.phones.clone() {
            phones.append(&mut self_phones);
        } else {
            self.phones = (!phones.is_empty()).then_some(phones);
        }
        self
    }

    /// Add a facsimile phone to the set of phones.
    pub fn add_fax_phones(mut self, phones: &[impl ToString]) -> Self {
        let mut phones: Vec<Phone> = phones
            .iter()
            .map(|p| {
                Phone::builder()
                    .contexts(vec!["fax".to_string()])
                    .phone(p.to_string())
                    .build()
            })
            .collect();
        if let Some(mut self_phones) = self.phones.clone() {
            phones.append(&mut self_phones);
        } else {
            self.phones = (!phones.is_empty()).then_some(phones);
        }
        self
    }

    /// Set the set of postal addresses to only be the passed in postal address.
    pub fn set_postal_address(mut self, postal_address: PostalAddress) -> Self {
        self.postal_addresses = Some(vec![postal_address]);
        self
    }
}

/// The language preference of the contact.
#[derive(Debug, Builder, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lang {
    /// The ordinal of the preference for this language.
    pub preference: Option<u64>,

    /// RFC 5646 language tag.
    pub tag: String,
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pref) = self.preference {
            write!(f, "{} (pref: {})", self.tag, pref)
        } else {
            f.write_str(&self.tag)
        }
    }
}

/// Name parts of a name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NameParts {
    /// Name prefixes.
    pub prefixes: Option<Vec<String>>,

    /// Surnames or last names.
    pub surnames: Option<Vec<String>>,

    /// Middle names.
    pub middle_names: Option<Vec<String>>,

    /// Given or first names.
    pub given_names: Option<Vec<String>>,

    /// Name suffixes.
    pub suffixes: Option<Vec<String>>,
}

#[buildstructor::buildstructor]
impl NameParts {
    #[builder(visibility = "pub")]
    fn new(
        prefixes: Vec<String>,
        surnames: Vec<String>,
        middle_names: Vec<String>,
        given_names: Vec<String>,
        suffixes: Vec<String>,
    ) -> Self {
        Self {
            prefixes: to_opt_vec(prefixes),
            surnames: to_opt_vec(surnames),
            middle_names: to_opt_vec(middle_names),
            given_names: to_opt_vec(given_names),
            suffixes: to_opt_vec(suffixes),
        }
    }
}

/// A postal address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostalAddress {
    /// Preference of this address in relation to others.
    pub preference: Option<u64>,

    /// Work, home, etc.... Known as "type" in JCard.
    pub contexts: Option<Vec<String>>,

    /// An unstructured address. An unstructured postal address is
    /// usually the complete postal address. That is, this string
    /// would contain the street address, country, region, postal code, etc...
    ///
    /// Depending on how the postal address is given, it can either
    /// be structured or unstructured. If it is given as unstructured,
    /// then this value is populated.
    ///
    /// It is possible that a single postal address is given as both,
    /// in which case this value is populated along with the other
    /// values of the postal address.   
    pub full_address: Option<String>,

    /// Invidual street lines.
    pub street_parts: Option<Vec<String>>,

    /// City name, county name, etc...
    pub locality: Option<String>,

    /// Name of region (i.e. state, province, etc...).
    pub region_name: Option<String>,

    /// Code for region.
    pub region_code: Option<String>,

    /// Name of the country.
    pub country_name: Option<String>,

    /// Code of the country.
    pub country_code: Option<String>,

    /// Postal code.
    pub postal_code: Option<String>,
}

#[buildstructor::buildstructor]
impl PostalAddress {
    #[builder(visibility = "pub")]
    fn new(
        preference: Option<u64>,
        contexts: Vec<String>,
        full_address: Option<String>,
        street_parts: Vec<String>,
        locality: Option<String>,
        region_name: Option<String>,
        region_code: Option<String>,
        country_name: Option<String>,
        country_code: Option<String>,
        postal_code: Option<String>,
    ) -> Self {
        Self {
            preference,
            contexts: to_opt_vec(contexts),
            full_address,
            street_parts: to_opt_vec(street_parts),
            locality,
            region_name,
            region_code,
            country_name,
            country_code,
            postal_code,
        }
    }
}

/// Represents an email address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Email {
    /// Preference of this email in relation to others.
    pub preference: Option<u64>,

    /// Work, home, etc.... Known as "type" in JCard.
    pub contexts: Option<Vec<String>>,

    /// The email address.
    pub email: String,
}

#[buildstructor::buildstructor]
impl Email {
    #[builder(visibility = "pub")]
    fn new(preference: Option<u64>, contexts: Vec<String>, email: String) -> Self {
        Self {
            preference,
            contexts: to_opt_vec(contexts),
            email,
        }
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut qualifiers = vec![];
        if let Some(pref) = self.preference {
            qualifiers.push(format!("(pref: {pref})"));
        }
        if let Some(contexts) = &self.contexts {
            qualifiers.push(format!("({})", contexts.join(",")));
        }
        let qualifiers = qualifiers.join(" ");
        if qualifiers.is_empty() {
            f.write_str(&self.email)
        } else {
            write!(f, "{} {}", &self.email, qualifiers)
        }
    }
}

/// Represents phone number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Phone {
    /// Preference of this phone in relation to others.
    pub preference: Option<u64>,

    /// Work, home, etc.... Known as "type" in JCard.
    pub contexts: Option<Vec<String>>,

    /// The phone number.
    pub phone: String,

    /// Features (voice, fax, etc...)
    pub features: Option<Vec<String>>,
}

#[buildstructor::buildstructor]
impl Phone {
    #[builder(visibility = "pub")]
    fn new(
        preference: Option<u64>,
        contexts: Vec<String>,
        phone: String,
        features: Vec<String>,
    ) -> Self {
        Self {
            preference,
            contexts: to_opt_vec(contexts),
            phone,
            features: to_opt_vec(features),
        }
    }
}

impl Display for Phone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut qualifiers = vec![];
        if let Some(pref) = self.preference {
            qualifiers.push(format!("(pref: {pref})"));
        }
        if let Some(contexts) = &self.contexts {
            qualifiers.push(format!("({})", contexts.join(",")));
        }
        if let Some(features) = &self.features {
            qualifiers.push(format!("({})", features.join(",")));
        }
        let qualifiers = qualifiers.join(" ");
        if qualifiers.is_empty() {
            f.write_str(&self.phone)
        } else {
            write!(f, "{} {}", &self.phone, qualifiers)
        }
    }
}
