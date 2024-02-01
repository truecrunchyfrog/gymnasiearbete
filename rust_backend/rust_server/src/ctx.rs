use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: Uuid,
}

// Constructor.
impl Ctx {
    pub const fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

// Property Accessors.
impl Ctx {
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }
}
