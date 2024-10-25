use std::future::Future;

use moka::future::{Cache, CacheBuilder};
use poise::serenity_prelude::CreateEmbed;

/// A simple counter that simulates swiping pages
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

/// A paginator that uses a generator function to generate embeds dynamically
#[derive(Debug)]
pub struct PaginateEmbedsLazily<S, Gen> {
    state: S,
    generator: Gen,
    current_index: usize,
    length: usize,
    cache: Cache<usize, CreateEmbed>,
}

impl<'a, S, Fut, Gen> PaginateEmbedsLazily<S, Gen>
where
    S: Clone + Send + Sync,
    Fut: Future<Output = Option<CreateEmbed>>,
    Gen: Fn(S, usize) -> Fut,
{
    pub fn new(length: usize, generator: Gen, state: S) -> Self {
        Self {
            state,
            generator,
            current_index: 0,
            length,
            cache: CacheBuilder::new(10).build(),
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

    pub async fn current_page(&self) -> Option<CreateEmbed> {
        (self.generator)(self.state.clone(), self.current_index).await
    }

    async fn fetch(&mut self, at: usize) -> Option<CreateEmbed> {
        self.cache
            .entry(at)
            .or_optionally_insert_with(async { (self.generator)(self.state.clone(), at).await })
            .await
            .map(|value| value.into_value())
    }

    async fn fetch_and_set(&mut self, at: usize) -> Option<CreateEmbed> {
        match self.fetch(at).await {
            Some(t) => {
                self.current_index = at;
                Some(t)
            },
            None => None,
        }
    }

    /// Increments the internal pointer by 1.
    /// If this pointer exceeds the length of the slice, it won't increment and will return [None].
    pub async fn next_page(&mut self) -> Option<CreateEmbed> {
        self.fetch_and_set(self.current_index + 1).await
    }

    /// Decrements the internal pointer by 1.
    /// If this pointer exceeds the length of the slice, it won't decrement and will return [None].
    pub async fn previous_page(&mut self) -> Option<CreateEmbed> {
        self.fetch_and_set(self.current_index - 1).await
    }

    pub async fn first_page(&mut self) -> Option<CreateEmbed> {
        self.fetch_and_set(0).await
    }

    pub async fn last_page(&mut self) -> Option<CreateEmbed> {
        self.fetch_and_set(self.length - 1).await
    }

    pub async fn jump_to(&mut self, to: usize) -> Option<CreateEmbed> {
        self.fetch_and_set(to).await
    }
}
