use super::*;
use crate::core::{usecases, util::filter::InBBox};
use std::result;

//TODO: move tests to corresponding usecase

type RepoResult<T> = result::Result<T, RepoError>;

trait Id {
    fn id(&self) -> &str;
}

impl Id for Entry {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Event {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Category {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Tag {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for User {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Comment {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Rating {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for BboxSubscription {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Organization {
    fn id(&self) -> &str {
        &self.id
    }
}

pub struct MockDb {
    pub entries: Vec<Entry>,
    pub events: Vec<Event>,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub users: Vec<User>,
    pub ratings: Vec<Rating>,
    pub comments: Vec<Comment>,
    pub bbox_subscriptions: Vec<BboxSubscription>,
    pub orgs: Vec<Organization>,
}

impl MockDb {
    pub fn new() -> MockDb {
        MockDb {
            entries: vec![],
            events: vec![],
            categories: vec![],
            tags: vec![],
            users: vec![],
            ratings: vec![],
            comments: vec![],
            bbox_subscriptions: vec![],
            orgs: vec![],
        }
    }
}

fn get<T: Clone + Id>(objects: &[T], id: &str) -> RepoResult<T> {
    match objects.iter().find(|x| x.id() == id) {
        Some(x) => Ok(x.clone()),
        None => Err(RepoError::NotFound),
    }
}

fn create<T: Clone + Id>(objects: &mut Vec<T>, e: T) -> RepoResult<()> {
    if objects.iter().any(|x| x.id() == e.id()) {
        return Err(RepoError::AlreadyExists);
    } else {
        objects.push(e);
    }
    Ok(())
}

fn update<T: Clone + Id>(objects: &mut Vec<T>, e: &T) -> RepoResult<()> {
    if let Some(pos) = objects.iter().position(|x| x.id() == e.id()) {
        objects[pos] = e.clone();
    } else {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

fn delete<T: Clone + Id>(objects: &mut Vec<T>, id: &str) -> RepoResult<()> {
    if let Some(pos) = objects.iter().position(|x| x.id() == id) {
        objects.remove(pos);
    } else {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

impl EntryGateway for MockDb {
    fn create_entry(&mut self, e: Entry) -> RepoResult<()> {
        create(&mut self.entries, e)
    }
    fn get_entry(&self, id: &str) -> RepoResult<Entry> {
        get(&self.entries, id)
    }
    fn all_entries(&self) -> RepoResult<Vec<Entry>> {
        Ok(self.entries.clone())
    }

    fn get_entries_by_bbox(&self, bbox: &Bbox) -> RepoResult<Vec<Entry>> {
        Ok(self
            .entries
            .iter()
            .filter(|e| e.in_bbox(bbox))
            .cloned()
            .collect())
    }
    fn update_entry(&mut self, e: &Entry) -> RepoResult<()> {
        update(&mut self.entries, e)
    }

    fn import_multiple_entries(&mut self, entries: &[Entry]) -> RepoResult<()> {
        for e in entries.iter() {
            self.create_entry(e.clone())?;
            for t in e.tags.iter() {
                self.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
            }
        }
        Ok(())
    }
}

impl EventGateway for MockDb {
    fn create_event(&mut self, e: Event) -> RepoResult<()> {
        create(&mut self.events, e)
    }

    fn get_event(&self, id: &str) -> RepoResult<Event> {
        get(&self.events, id)
    }
    fn all_events(&self) -> RepoResult<Vec<Event>> {
        Ok(self.events.clone())
    }
    fn update_event(&mut self, e: &Event) -> RepoResult<()> {
        update(&mut self.events, e)
    }
    fn delete_event(&mut self, id: &str) -> RepoResult<()> {
        delete(&mut self.events, id)
    }
}

impl UserGateway for MockDb {
    fn create_user(&mut self, u: User) -> RepoResult<()> {
        create(&mut self.users, u)
    }

    fn get_user(&self, username: &str) -> RepoResult<User> {
        let users: &Vec<User> = &self
            .users
            .iter()
            .filter(|u| u.username == username)
            .cloned()
            .collect();
        if users.len() > 0 {
            Ok(users[0].clone())
        } else {
            Err(RepoError::NotFound)
        }
    }

    fn all_users(&self) -> RepoResult<Vec<User>> {
        Ok(self.users.clone())
    }

    fn delete_user(&mut self, u_id: &str) -> RepoResult<()> {
        self.users = self
            .users
            .clone()
            .into_iter()
            .filter(|u| u.id != u_id)
            .collect();
        Ok(())
    }

    fn update_user(&mut self, u: &User) -> RepoResult<()> {
        update(&mut self.users, u)
    }
}

impl CommentGateway for MockDb {
    fn create_comment(&mut self, c: Comment) -> RepoResult<()> {
        create(&mut self.comments, c)
    }

    fn all_comments(&self) -> RepoResult<Vec<Comment>> {
        Ok(self.comments.clone())
    }
}

impl OrganizationGateway for MockDb {
    fn create_org(&mut self, o: Organization) -> RepoResult<()> {
        create(&mut self.orgs, o)
    }
    fn get_org_by_api_token(&self, token: &str) -> RepoResult<Organization> {
        let o = self
            .orgs
            .iter()
            .find(|o| o.api_token == token)
            .ok_or(RepoError::NotFound)?;
        Ok(o.clone())
    }
    fn get_all_tags_owned_by_orgs(&self) -> RepoResult<Vec<String>> {
        Ok(self
            .orgs
            .iter()
            .flat_map(|o| o.owned_tags.clone())
            .collect())
    }
}

impl Db for MockDb {
    fn create_tag_if_it_does_not_exist(&mut self, e: &Tag) -> RepoResult<()> {
        if let Err(err) = create(&mut self.tags, e.clone()) {
            match err {
                RepoError::AlreadyExists => {
                    // that's ok
                }
                _ => return Err(err),
            }
        }
        Ok(())
    }

    fn create_category_if_it_does_not_exist(&mut self, e: &Category) -> RepoResult<()> {
        if let Err(err) = create(&mut self.categories, e.clone()) {
            match err {
                RepoError::AlreadyExists => {
                    // that's ok
                }
                _ => return Err(err),
            }
        }
        Ok(())
    }

    fn create_rating(&mut self, r: Rating) -> RepoResult<()> {
        create(&mut self.ratings, r)
    }

    fn create_bbox_subscription(&mut self, s: &BboxSubscription) -> RepoResult<()> {
        create(&mut self.bbox_subscriptions, s.clone())
    }

    fn all_categories(&self) -> RepoResult<Vec<Category>> {
        Ok(self.categories.clone())
    }

    fn all_tags(&self) -> RepoResult<Vec<Tag>> {
        Ok(self.tags.clone())
    }

    fn all_ratings(&self) -> RepoResult<Vec<Rating>> {
        Ok(self.ratings.clone())
    }

    fn all_bbox_subscriptions(&self) -> RepoResult<Vec<BboxSubscription>> {
        Ok(self.bbox_subscriptions.clone())
    }

    fn delete_bbox_subscription(&mut self, s_id: &str) -> RepoResult<()> {
        self.bbox_subscriptions = self
            .bbox_subscriptions
            .iter()
            .filter(|s| s.id != s_id)
            .cloned()
            .collect();
        Ok(())
    }
}

#[test]
fn receive_different_user() {
    let mut db = MockDb::new();
    db.users = vec![
        User {
            id: "1".into(),
            username: "a".into(),
            password: "a".into(),
            email: "a@foo.bar".into(),
            email_confirmed: true,
            role: Role::Guest,
        },
        User {
            id: "2".into(),
            username: "b".into(),
            password: "b".into(),
            email: "b@foo.bar".into(),
            email_confirmed: true,
            role: Role::Guest,
        },
    ];
    assert!(get_user(&mut db, "a", "b").is_err());
    assert!(get_user(&mut db, "a", "a").is_ok());
}

#[test]
fn create_bbox_subscription() {
    let mut db = MockDb::new();
    let bbox_new = Bbox {
        north_east: Coordinate {
            lat: 10.0,
            lng: 10.0,
        },
        south_west: Coordinate {
            lat: 10.0,
            lng: 5.0,
        },
    };

    let username = "a";
    assert!(db
        .create_user(User {
            id: "123".into(),
            username: username.into(),
            password: username.into(),
            email: "abc@abc.de".into(),
            email_confirmed: true,
            role: Role::Guest,
        })
        .is_ok());
    assert!(usecases::subscribe_to_bbox(
        &vec![bbox_new.south_west, bbox_new.north_east],
        username.into(),
        &mut db,
    )
    .is_ok());

    let bbox_subscription = db.all_bbox_subscriptions().unwrap()[0].clone();
    assert_eq!(bbox_subscription.bbox.north_east.lat, 10.0);
}

#[test]
fn modify_bbox_subscription() {
    let mut db = MockDb::new();

    let bbox_old = Bbox {
        north_east: Coordinate {
            lat: 50.0,
            lng: 10.0,
        },
        south_west: Coordinate {
            lat: 50.0,
            lng: 5.0,
        },
    };

    let bbox_new = Bbox {
        north_east: Coordinate {
            lat: 10.0,
            lng: 10.0,
        },
        south_west: Coordinate {
            lat: 10.0,
            lng: 5.0,
        },
    };

    let username = "a";
    assert!(db
        .create_user(User {
            id: "123".into(),
            username: username.into(),
            password: username.into(),
            email: "abc@abc.de".into(),
            email_confirmed: true,
            role: Role::Guest,
        })
        .is_ok());

    let bbox_subscription = BboxSubscription {
        id: "123".into(),
        bbox: bbox_old,
        username: "a".into(),
    };
    db.create_bbox_subscription(&bbox_subscription.clone())
        .unwrap();

    usecases::subscribe_to_bbox(
        &vec![bbox_new.south_west, bbox_new.north_east],
        username.into(),
        &mut db,
    )
    .unwrap();

    let bbox_subscriptions: Vec<_> = db
        .all_bbox_subscriptions()
        .unwrap()
        .into_iter()
        .filter(|s| &*s.username == "a")
        .collect();

    assert_eq!(bbox_subscriptions.len(), 1);
    assert_eq!(bbox_subscriptions[0].clone().bbox.north_east.lat, 10.0);
}

#[test]
fn get_bbox_subscriptions() {
    let mut db = MockDb::new();

    let bbox1 = Bbox {
        north_east: Coordinate {
            lat: 50.0,
            lng: 10.0,
        },
        south_west: Coordinate {
            lat: 50.0,
            lng: 5.0,
        },
    };

    let bbox2 = Bbox {
        north_east: Coordinate {
            lat: 10.0,
            lng: 10.0,
        },
        south_west: Coordinate {
            lat: 10.0,
            lng: 5.0,
        },
    };

    let user1 = "a";
    assert!(db
        .create_user(User {
            id: user1.into(),
            username: user1.into(),
            password: user1.into(),
            email: "abc@abc.de".into(),
            email_confirmed: true,
            role: Role::Guest,
        })
        .is_ok());
    let bbox_subscription = BboxSubscription {
        id: "1".into(),
        bbox: bbox1,
        username: "a".into(),
    };
    assert!(db
        .create_bbox_subscription(&bbox_subscription.clone())
        .is_ok());

    let user2 = "b";
    assert!(db
        .create_user(User {
            id: user2.into(),
            username: user2.into(),
            password: user2.into(),
            email: "abc@abc.de".into(),
            email_confirmed: true,
            role: Role::Guest,
        })
        .is_ok());
    let bbox_subscription2 = BboxSubscription {
        id: "2".into(),
        bbox: bbox2,
        username: "b".into(),
    };
    assert!(db
        .create_bbox_subscription(&bbox_subscription2.clone())
        .is_ok());
    let bbox_subscriptions = usecases::get_bbox_subscriptions(user2.into(), &mut db);
    assert!(bbox_subscriptions.is_ok());
    assert_eq!(bbox_subscriptions.unwrap()[0].id, "2");
}

#[test]
fn email_addresses_by_coordinate() {
    let mut db = MockDb::new();
    let bbox_new = Bbox {
        north_east: Coordinate {
            lat: 10.0,
            lng: 10.0,
        },
        south_west: Coordinate { lat: 0.0, lng: 0.0 },
    };

    let username = "a";
    let u_id = "123".to_string();
    db.create_user(User {
        id: u_id.clone(),
        username: username.into(),
        password: "123".into(),
        email: "abc@abc.de".into(),
        email_confirmed: true,
        role: Role::Guest,
    })
    .unwrap();

    usecases::subscribe_to_bbox(
        &vec![bbox_new.south_west, bbox_new.north_east],
        username,
        &mut db,
    )
    .unwrap();

    let email_addresses = usecases::email_addresses_by_coordinate(&mut db, &5.0, &5.0).unwrap();
    assert_eq!(email_addresses.len(), 1);
    assert_eq!(email_addresses[0], "abc@abc.de");

    let no_email_addresses =
        usecases::email_addresses_by_coordinate(&mut db, &20.0, &20.0).unwrap();
    assert_eq!(no_email_addresses.len(), 0);
}

#[test]
fn delete_user() {
    let mut db = MockDb::new();
    let username = "a".to_string();
    let u_id = "1".to_string();
    assert!(db
        .create_user(User {
            id: u_id.clone(),
            username: username.clone(),
            password: username,
            email: "abc@abc.de".into(),
            email_confirmed: true,
            role: Role::Guest,
        })
        .is_ok());
    let username = "b".to_string();
    let u_id = "2".to_string();
    assert!(db
        .create_user(User {
            id: u_id.clone(),
            username: username.clone(),
            password: username,
            email: "abcd@abcd.de".into(),
            email_confirmed: true,
            role: Role::Guest,
        })
        .is_ok());
    assert_eq!(db.users.len(), 2);

    assert!(usecases::delete_user(&mut db, "1", "1").is_ok());
    assert_eq!(db.users.len(), 1);
}

#[test]
fn receive_event_with_creators_email() {
    let mut db = MockDb::new();
    db.create_user(User {
        id: "x".into(),
        username: "user".into(),
        password: "pw".into(),
        email: "abc@abc.de".into(),
        email_confirmed: true,
        role: Role::Guest,
    })
    .unwrap();
    db.create_event(Event {
        id: "x".into(),
        title: "t".into(),
        description: None,
        start: 0,
        end: None,
        contact: None,
        location: None,
        homepage: None,
        tags: vec![],
        created_by: Some("user".into()),
    })
    .unwrap();
    let e = usecases::get_event(&mut db, "x").unwrap();
    assert_eq!(e.created_by.unwrap(), "abc@abc.de");
}
