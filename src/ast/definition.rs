use super::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Definition(HashMap<Query, Body>);
