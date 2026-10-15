use std::collections::{HashMap, VecDeque};

/// One discarded cache entry. The key stays inside the cache so callers cannot log it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtworkEviction {
    pub bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtworkInsert {
    pub stored: bool,
    pub evicted: Vec<ArtworkEviction>,
}

#[derive(Debug)]
struct Entry<T> {
    bytes: usize,
    value: T,
}

/// LRU of decoded artwork. `budget_bytes` is a hard cap on retained pixel buffers.
#[derive(Debug)]
pub struct ArtworkCache<T> {
    budget_bytes: usize,
    used_bytes: usize,
    entries: HashMap<String, Entry<T>>,
    order: VecDeque<String>,
}

impl<T> ArtworkCache<T> {
    pub fn new(budget_bytes: usize) -> Self {
        Self {
            budget_bytes,
            used_bytes: 0,
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn peek(&self, key: &str) -> Option<&T> {
        self.entries.get(key).map(|entry| &entry.value)
    }

    pub fn touch(&mut self, key: &str) {
        if self.entries.contains_key(key) {
            self.order.retain(|existing| existing != key);
            self.order.push_back(key.to_owned());
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.used_bytes = 0;
    }

    /// Insert `key`, evicting the oldest entries until it fits.
    ///
    /// `protected` is never evicted. An item larger than the budget is refused
    /// without discarding anything already retained.
    pub fn insert(
        &mut self,
        key: String,
        value: T,
        bytes: usize,
        protected: Option<&str>,
    ) -> ArtworkInsert {
        if bytes > self.budget_bytes {
            return ArtworkInsert {
                stored: false,
                evicted: Vec::new(),
            };
        }

        let replaced = self.entries.get(&key).map_or(0, |entry| entry.bytes);
        let mut occupied = self.used_bytes.saturating_sub(replaced);
        let mut planned_indexes = Vec::new();
        while occupied.saturating_add(bytes) > self.budget_bytes {
            let Some(index) = self
                .order
                .iter()
                .enumerate()
                .position(|(index, candidate)| {
                    candidate != &key
                        && protected.is_none_or(|keep| candidate != keep)
                        && !planned_indexes.contains(&index)
                })
            else {
                return ArtworkInsert {
                    stored: false,
                    evicted: Vec::new(),
                };
            };
            let victim = &self.order[index];
            let Some(victim_bytes) = self.entries.get(victim).map(|entry| entry.bytes) else {
                return ArtworkInsert {
                    stored: false,
                    evicted: Vec::new(),
                };
            };
            occupied = occupied.saturating_sub(victim_bytes);
            planned_indexes.push(index);
        }

        let mut evicted = Vec::new();
        planned_indexes.sort_unstable_by(|left, right| right.cmp(left));
        for index in planned_indexes {
            if let Some(removed) = self.remove_at(index) {
                evicted.push(removed);
            }
        }
        if let Some(existing) = self.entries.remove(&key) {
            self.used_bytes = self.used_bytes.saturating_sub(existing.bytes);
            self.order.retain(|existing_key| existing_key != &key);
        }

        self.used_bytes = self.used_bytes.saturating_add(bytes);
        self.order.push_back(key.clone());
        self.entries.insert(key, Entry { bytes, value });
        ArtworkInsert {
            stored: true,
            evicted,
        }
    }

    fn remove_at(&mut self, index: usize) -> Option<ArtworkEviction> {
        let key = self.order.remove(index)?;
        let entry = self.entries.remove(&key)?;
        self.used_bytes = self.used_bytes.saturating_sub(entry.bytes);
        Some(ArtworkEviction { bytes: entry.bytes })
    }
}

#[cfg(test)]
#[path = "__tests__/artwork_cache.rs"]
mod tests;
