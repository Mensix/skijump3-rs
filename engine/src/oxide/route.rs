#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavAction<R> {
    None,
    Navigate(R),
    Back,
    Quit,
}
