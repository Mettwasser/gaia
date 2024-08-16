use std::future::Future;

pub struct Paginate<'a, T> {
    paginate_on: &'a [T],
    current_index: usize,
}

impl<'a, T> Paginate<'a, T> {
    pub fn new(paginate_on: &'a [T]) -> Self {
        Self {
            paginate_on,
            current_index: 0,
        }
    }

    pub fn max(&self) -> usize {
        self.paginate_on.len()
    }

    pub fn current_idx(&self) -> usize {
        self.current_index
    }

    pub fn current_page(&self) -> Option<&'a T> {
        self.paginate_on.get(self.current_idx())
    }

    /// Increments the internal pointer by 1.
    /// If this pointer exceeds the length of the slice, it won't increment and will return [None].
    pub fn next_page(&mut self) -> Option<&'a T> {
        let next_index = self.current_index + 1;
        if self.paginate_on.len() <= next_index {
            return None;
        }
        self.current_index = next_index;

        Some(self.paginate_on.get(self.current_index).unwrap())
    }

    /// Decrements the internal pointer by 1.
    /// If this pointer exceeds the length of the slice, it won't decrement and will return [None].
    pub fn previous_page(&mut self) -> Option<&'a T> {
        let previous = self.current_index - 1;
        if self.paginate_on.len() <= previous {
            return None;
        }
        self.current_index = previous;

        Some(self.paginate_on.get(self.current_index).unwrap())
    }

    pub fn first_page(&mut self) -> Option<&'a T> {
        self.current_index = 0;
        self.paginate_on.get(self.current_index)
    }

    pub fn last_page(&mut self) -> Option<&'a T> {
        self.current_index = self.paginate_on.len() - 1;
        self.paginate_on.get(self.current_index)
    }
}

#[derive(Debug)]
pub struct PaginateLazily<S, Gen> {
    state: S,
    generator: Gen,
    current_index: usize,
    length: usize,
}

impl<'a, S, Fut, Gen, T> PaginateLazily<S, Gen>
where
    S: Clone + Send + Sync,
    Fut: Future<Output = Option<T>>,
    Gen: Fn(S, usize) -> Fut,
{
    pub fn new(length: usize, generator: Gen, state: S) -> Self {
        Self {
            state,
            generator,
            current_index: 0,
            length,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn current_idx(&self) -> usize {
        self.current_index
    }

    pub async fn current_page(&self) -> Option<T> {
        (self.generator)(self.state.clone(), self.current_index).await
    }

    async fn fetch(&mut self, at: usize) -> Option<T> {
        match (self.generator)(self.state.clone(), at).await {
            Some(t) => {
                self.current_index = at;
                Some(t)
            }
            None => None,
        }
    }

    /// Increments the internal pointer by 1.
    /// If this pointer exceeds the length of the slice, it won't increment and will return [None].
    pub async fn next_page(&mut self) -> Option<T> {
        self.fetch(self.current_index + 1).await
    }

    /// Decrements the internal pointer by 1.
    /// If this pointer exceeds the length of the slice, it won't decrement and will return [None].
    pub async fn previous_page(&mut self) -> Option<T> {
        self.fetch(self.current_index - 1).await
    }

    pub async fn first_page(&mut self) -> Option<T> {
        self.fetch(0).await
    }

    pub async fn last_page(&mut self) -> Option<T> {
        self.fetch(self.length - 1).await
    }

    pub async fn jump_to(&mut self, to: usize) -> Option<T> {
        self.fetch(to).await
    }
}
