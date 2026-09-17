#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriComponent {
    label: String
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri {
    components: Vec<UriComponent>
}