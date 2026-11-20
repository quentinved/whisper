use uuid::Uuid;

/// Sender-only capability to read a secret's delivery status. It can never
/// open, decrypt or burn the secret it tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusToken {
    value: Uuid,
}

impl StatusToken {
    pub fn new(value: Uuid) -> Self {
        Self { value }
    }

    pub fn generate() -> Self {
        Self {
            value: Uuid::new_v4(),
        }
    }

    pub fn value(&self) -> Uuid {
        self.value
    }
}

impl TryFrom<&str> for StatusToken {
    type Error = uuid::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            value: Uuid::parse_str(s)?,
        })
    }
}

impl std::fmt::Display for StatusToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
