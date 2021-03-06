use super::try_into_new_event;
use crate::core::prelude::*;

pub use super::NewEvent as UpdateEvent;

pub fn update_event<D: Db>(db: &mut D, id: &str, e: UpdateEvent) -> Result<()> {
    let mut updated_event = try_into_new_event(db, e)?;
    debug!("Updating event: {:?}", updated_event);
    updated_event.id = id.into();
    db.update_event(&updated_event)?;
    Ok(())
}
