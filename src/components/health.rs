pub struct Health {
    pub(crate) current: i32,
    pub(crate) max: i32,
}

impl Health {
    pub(crate) fn new(max: i32) -> Self {
        Health { current: max, max }
    }
}