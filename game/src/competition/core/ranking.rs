#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ranked<T> {
    pub item: T,
    pub rank: usize,
    pub score: i32,
}

#[must_use]
pub fn ranked_order<T>(
    items: impl IntoIterator<Item = T>,
    score: impl Fn(T) -> i32,
) -> Vec<Ranked<T>>
where
    T: Copy,
{
    let mut ranked: Vec<Ranked<T>> = items
        .into_iter()
        .map(|item| Ranked {
            item,
            rank: 0,
            score: score(item),
        })
        .collect();
    ranked.sort_by(|a, b| b.score.cmp(&a.score));

    let mut rank = 1;
    for i in 0..ranked.len() {
        if i > 0 && ranked[i].score < ranked[i - 1].score {
            rank = i + 1;
        }
        ranked[i].rank = rank;
    }
    ranked
}
