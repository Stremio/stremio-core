use crate::state_types::Event::*;
use crate::state_types::Internal::*;
use crate::types::LibItem;
use crate::types::api::*;
use std::collections::HashMap;

pub type LibraryIndex = HashMap<String, LibItem>;
