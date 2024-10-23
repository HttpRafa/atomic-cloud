use url::Url;

pub struct Profiles {
    pub profiles: Vec<Profile>,
}

impl Profiles {
    fn new(profiles: Vec<Profile>) -> Self {
        Profiles {
            profiles,
        }
    }

    pub fn load_all() -> Self {
        Self::new(vec![])
    }
}

pub struct Profile {
    pub name: String,
    pub url: Url,
}