use crate::modules::{ConnectionType, EventHandler};

pub fn empty_write(handler: &mut EventHandler, connection: &ConnectionType, data: &[u8]) {}
