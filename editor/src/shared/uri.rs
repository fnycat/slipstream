pub struct Uri {
    components: Vec<String>,
}

impl Uri {
    pub fn name(&self) -> Option<&str> {
        self.components.last().map(String::as_ref)
    }

    pub fn head(&self) -> Option<&str> {
        self.components.first().map(String::as_ref)
    }

    pub fn as_slice(&self) -> UriSlice {
        UriSlice {
            components: &self.components,
        }
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

    pub fn name(&self) -> Option<&str> {
        self.components.last().map(String::as_ref)
    }
}
