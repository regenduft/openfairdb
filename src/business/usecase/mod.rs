use super::error::{Error, RepoError};
use std::result;
use chrono::*;
use entities::*;
use super::db::Db;
use super::filter;
use super::validate::{self, Validate};
use uuid::Uuid;
use std::collections::HashMap;
use pwhash::bcrypt;

#[cfg(test)]
pub mod tests;

type Result<T> = result::Result<T,Error>;

trait Id {
    fn id(&self) -> String;
}

impl Id for Entry {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for Category {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for Tag {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for User {
    fn id(&self) -> String {
        self.username.clone()
    }
}

fn triple_id(t: &Triple) -> String {
    let (s_type, s_id) = match t.subject {
        ObjectId::Entry(ref id) => ("entry", id),
        ObjectId::Tag(ref id) => ("tag", id),
        ObjectId::User(ref id) => ("user", id)
    };
    let (o_type, o_id) = match t.object {
        ObjectId::Entry(ref id) => ("entry", id),
        ObjectId::Tag(ref id) => ("tag", id),
        ObjectId::User(ref id) => ("user", id)
    };
    let p_type = match t.predicate {
        Relation::IsTaggedWith => "is_tagged_with"
    };
    format!("{}-{}-{}-{}-{}",s_type,s_id,p_type,o_type,o_id)
}

impl Id for Triple {
    fn id(&self) -> String {
        triple_id(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewEntry {
    title       : String,
    description : String,
    lat         : f64,
    lng         : f64,
    street      : Option<String>,
    zip         : Option<String>,
    city        : Option<String>,
    country     : Option<String>,
    email       : Option<String>,
    telephone   : Option<String>,
    homepage    : Option<String>,
    categories  : Vec<String>,
    tags        : Vec<String>,
    license     : String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NewUser {
    username: String,
    password: String,
    email: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateEntry {
    id          : String,
    version     : u64,
    title       : String,
    description : String,
    lat         : f64,
    lng         : f64,
    street      : Option<String>,
    zip         : Option<String>,
    city        : Option<String>,
    country     : Option<String>,
    email       : Option<String>,
    telephone   : Option<String>,
    homepage    : Option<String>,
    categories  : Vec<String>,
    tags        : Vec<String>,
}

fn create_missing_tags<D:Db>(db: &mut D, tags: &[String]) -> Result<()> {
    let existing_tags = db.all_tags()?;
    for new_t in tags {
        if !existing_tags.iter().any(|t|t.id == *new_t){
            db.create_tag(&Tag{id:new_t.clone()})?;
        }
    }
    Ok(())
}

struct Diff<T> {
    new: Vec<T>,
    deleted: Vec<T>
}

fn get_triple_diff(old: &[Triple], new: &[Triple]) -> Diff<Triple> {

    let to_create = new
        .iter()
        .filter(|t|!old.iter().any(|x| x == *t))
        .cloned()
        .collect::<Vec<Triple>>();

    let to_delete = old
        .iter()
        .filter(|t|!new.iter().any(|x| x == *t))
        .cloned()
        .collect::<Vec<Triple>>();

    Diff{
        new: to_create,
        deleted: to_delete
    }
}


fn set_tag_relations<D:Db>(db: &mut D, entry: &str, tags: &[String]) -> Result<()> {
    create_missing_tags(db, tags)?;
    let subject = ObjectId::Entry(entry.into());
    let old_triples = db.all_triples()?
        .into_iter()
        .filter(|x|x.subject == subject)
        .filter(|x|x.predicate == Relation::IsTaggedWith)
        .collect::<Vec<Triple>>();
    let new_triples = tags
        .into_iter()
        .map(|x| Triple{
            subject: subject.clone(),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag(x.clone())
        })
        .collect::<Vec<Triple>>();

    let diff = get_triple_diff(&old_triples, &new_triples);

    for t in diff.new {
        db.create_triple(&t)?;
    }
    for t in diff.deleted {
        db.delete_triple(&t)?;
    }
    Ok(())
}

pub fn get_tag_ids<D:Db>(db: &D) -> Result<Vec<String>> {
    let mut tags : Vec<String> = db
        .all_triples()?
        .into_iter()
        .filter(|t|t.predicate == Relation::IsTaggedWith)
        .filter_map(|t| match t.object {
           ObjectId::Tag(id) => Some(id),
            _ => None
        })
        .collect();
    tags.dedup();
    Ok(tags)
}

pub fn get_tag_ids_for_entry_id(triples: &[Triple], entry_id : &str) -> Vec<Tag> {
    triples
        .iter()
        .filter(&*filter::triple_by_entry_id(entry_id))
        .filter(|triple| triple.predicate == Relation::IsTaggedWith)
        .map(|triple|&triple.object)
        .filter_map(|object|
            match *object {
                ObjectId::Tag(ref tag_id) => Some(tag_id),
                _ => None
            })
        .cloned()
        .map(|tag_id|Tag{id: tag_id})
        .collect()
}

pub fn get_tags_by_entry_ids<D:Db>(db : &D, ids : &[String]) -> Result<HashMap<String, Vec<Tag>>> {
    let triples = db.all_triples()?;
    Ok(ids
        .iter()
        .map(|id|(
            id.clone(),
            get_tag_ids_for_entry_id(&triples, id)
        ))
        .collect())
}

pub fn get_entries<D:Db>(db : &D, ids : &[String]) -> Result<Vec<Entry>> {
    let entries = db
        .all_entries()?
        .into_iter()
        .filter(|e|ids.iter().any(|id| *id == e.id))
        .collect();
    Ok(entries)
}

pub fn create_new_user<D: Db>(db: &mut D, u: NewUser) -> Result<()> {
    validate::username(&u.username)?;
    validate::password(&u.password)?;
    validate::email(&u.email)?;
    let pw = bcrypt::hash(&u.password)?;
    db.create_user(&User{
        username: u.username,
        password: pw,
        email: u.email,
    })?;
    Ok(())
}

pub fn create_new_entry<D: Db>(db: &mut D, e: NewEntry) -> Result<String>
 {
    let new_entry = Entry{
        id          :  Uuid::new_v4().simple().to_string(),
        created     :  UTC::now().timestamp() as u64,
        version     :  0,
        title       :  e.title,
        description :  e.description,
        lat         :  e.lat,
        lng         :  e.lng,
        street      :  e.street,
        zip         :  e.zip,
        city        :  e.city,
        country     :  e.country,
        email       :  e.email,
        telephone   :  e.telephone,
        homepage    :  e.homepage,
        categories  :  e.categories,
        license     :  Some(e.license)
    };
    new_entry.validate()?;
    db.create_entry(&new_entry)?;
    set_tag_relations(db, &new_entry.id, &e.tags)?;
    Ok(new_entry.id)
}

pub fn update_entry<D: Db>(db: &mut D, e: UpdateEntry) -> Result<()> {
    let old : Entry = db.get_entry(&e.id)?;
    if (old.version + 1) != e.version {
        return Err(Error::Repo(RepoError::InvalidVersion))
    }
    let new_entry = Entry{
        id          :  e.id,
        created     :  UTC::now().timestamp() as u64,
        version     :  e.version,
        title       :  e.title,
        description :  e.description,
        lat         :  e.lat,
        lng         :  e.lng,
        street      :  e.street,
        zip         :  e.zip,
        city        :  e.city,
        country     :  e.country,
        email       :  e.email,
        telephone   :  e.telephone,
        homepage    :  e.homepage,
        categories  :  e.categories,
        license     :  old.license
    };
    db.update_entry(&new_entry)?;
    set_tag_relations(db, &new_entry.id, &e.tags)?;
    Ok(())
}
