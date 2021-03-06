#[cfg(feature = "email")]
use super::mail;
use crate::{
    adapters::user_communication,
    core::{prelude::*, usecases},
};
use regex::Regex;

lazy_static! {
    static ref HASH_TAG_REGEX: Regex = Regex::new(r"#(?P<tag>\w+((-\w+)*)?)").unwrap();
}

pub const ID_LIST_SEPARATOR: char = ',';

pub fn extract_ids(s: &str) -> Vec<String> {
    s.split(ID_LIST_SEPARATOR)
        .map(|x| x.trim().to_owned())
        .filter(|id| id != "")
        .collect()
}

#[cfg(all(not(test), feature = "email"))]
fn send_mail(mail: String) {
    std::thread::spawn(move || {
        if let Err(err) = mail::sendmail::send(&mail) {
            warn!("Could not send e-mail: {}", err);
        }
    });
}

/// Don't actually send emails while running the tests
#[cfg(all(test, feature = "email"))]
fn send_mail(mail: String) {
    debug!("Would send e-mail: {}", mail);
}

#[cfg(feature = "email")]
pub fn send_mails(email_addresses: &[String], subject: &str, body: &str) {
    debug!("sending emails to: {:?}", email_addresses);
    for email_address in email_addresses.to_owned() {
        let to = vec![email_address];
        match mail::create(&to, subject, body) {
            Ok(mail) => {
                send_mail(mail);
            }
            Err(e) => {
                warn!("could not create notification mail: {}", e);
            }
        }
    }
}

pub fn notify_create_entry(
    email_addresses: &[String],
    e: &usecases::NewEntry,
    id: &str,
    all_categories: Vec<Category>,
) {
    let subject = String::from("Karte von morgen - neuer Eintrag: ") + &e.title;
    let categories: Vec<String> = all_categories
        .into_iter()
        .filter(|c| e.categories.clone().into_iter().any(|c_id| *c.id == c_id))
        .map(|c| c.name)
        .collect();
    let body = user_communication::new_entry_email(e, id, &categories);

    #[cfg(feature = "email")]
    send_mails(email_addresses, &subject, &body);
}

pub fn notify_update_entry(
    email_addresses: &[String],
    e: &usecases::UpdateEntry,
    all_categories: Vec<Category>,
) {
    let subject = String::from("Karte von morgen - Eintrag verändert: ") + &e.title;
    let categories: Vec<String> = all_categories
        .into_iter()
        .filter(|c| e.categories.clone().into_iter().any(|c_id| *c.id == c_id))
        .map(|c| c.name)
        .collect();
    let body = user_communication::changed_entry_email(e, &categories);

    #[cfg(feature = "email")]
    send_mails(email_addresses, &subject, &body);
}

pub fn extract_hash_tags(text: &str) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for cap in HASH_TAG_REGEX.captures_iter(text) {
        res.push(cap["tag"].into());
    }
    res
}

pub fn remove_hash_tags(text: &str) -> String {
    HASH_TAG_REGEX
        .replace_all(text, "")
        .into_owned()
        .replace("  ", " ")
        .replace(",", "")
        .trim()
        .into()
}
