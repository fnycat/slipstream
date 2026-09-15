pub struct Uri {
    components: Vec<String>,
}

impl Uri {
    pub fn tail(&self) -> &str {
        self.components.last().unwrap()
    }
}

pub struct UriSlice<'a> {
    components: &'a [String],
}

impl<'a> UriSlice<'a> {
    pub fn head(&self) -> Option<&str> {
        self.components.first().map(String::as_ref)
    }

    pub fn tail(self) -> Option<Self> {
        todo!()
    }
}
