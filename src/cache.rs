use std::{
    collections::HashMap,
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};

fn time_now() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis()
}

#[derive(PartialEq)]
pub enum CacheOption<T> {
    Value(T),
    Empty,
    Undefined,
    Expired,
}

impl<T> CacheOption<T> {
    pub fn unwrap(self) -> T {
        if let CacheOption::Value(value) = self {
            value
        } else {
            panic!("There is no value for unwrap.");
        }
    }

    pub fn unwrap_or(self, or: T) -> T {
        if let CacheOption::Value(value) = self {
            value
        } else {
            or
        }
    }

    pub fn is_value(&self) -> bool {
        match self {
            CacheOption::Value(_) => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            CacheOption::Empty => true,
            _ => false,
        }
    }

    pub fn is_undefined(&self) -> bool {
        match self {
            CacheOption::Undefined => true,
            _ => false,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self {
            CacheOption::Expired => true,
            _ => false,
        }
    }
}
#[derive(Default, Debug, PartialEq)]
struct CacheValue<T> {
    pub value: T,
    ttl_timestamp: u128,
}

impl<T> CacheValue<T> {
    pub fn new(value: T, ttl: u128) -> Self {
        Self {
            value,
            ttl_timestamp: ttl + time_now(),
        }
    }
}

#[derive(Default, Debug)]
struct Cache<T> {
    pub items: HashMap<String, Option<CacheValue<T>>>,
    ttl: u128,
}

impl<T> Cache<T>
where
    T: Default + Debug + PartialEq + Clone,
{
    pub fn new(ttl: u128) -> Self {
        Self {
            items: HashMap::default(),
            ttl,
        }
    }

    pub fn create(&mut self, key: String) {
        self.items.insert(key, None);
    }

    pub fn update(&mut self, key: &String, value: T) {
        if let Some(item) = self.items.get_mut(key) {
            *item = Some(CacheValue::new(value, self.ttl));
        } else {
            self.create(key.clone());
        }
    }

    pub fn get(self, key: &String) -> CacheOption<T> {
        match self.items.get(key) {
            Some(value) => match value {
                Some(cache_value) => {
                    if cache_value.ttl_timestamp >= time_now() {
                        CacheOption::Value(cache_value.value.clone())
                    } else {
                        CacheOption::Expired
                    }
                }
                None => CacheOption::Empty,
            },
            None => CacheOption::Undefined,
        }
    }

    pub fn remove(&mut self, key: &String) -> Option<Option<CacheValue<T>>> {
        self.items.remove(key)
    }

    pub fn clean(&mut self, key: &String) {
        if let Some(item) = self.items.get_mut(key) {
            *item = None;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::{thread::sleep, time::Duration};

    #[derive(Default, Debug, PartialEq, Clone)]
    struct DataTest {
        value: String,
    }

    #[test]
    fn value_undefined() {
        let cache = Cache::<DataTest>::new(1000);
        let key = "key".to_string();
        assert_eq!(cache.get(&key).is_undefined(), true);
    }

    #[test]
    fn value_empty() {
        let mut cache = Cache::<DataTest>::new(1000);
        let key = "key".to_string();
        cache.create(key.clone());

        assert_eq!(cache.get(&key).is_empty(), true);
    }

    #[test]
    fn value_found() {
        let mut cache = Cache::<DataTest>::new(1000);
        let key = "key".to_string();
        let value = DataTest::default();
        cache.create(key.clone());
        cache.update(&key, value.clone());

        assert_eq!(cache.get(&key).is_value(), true);
    }

    #[test]
    fn value_match() {
        let mut cache = Cache::<DataTest>::new(1000);
        let key = "key".to_string();
        let value = DataTest::default();
        cache.create(key.clone());
        cache.update(&key, value.clone());

        assert_eq!(cache.get(&key).unwrap(), value);
    }

    #[test]
    fn value_expired() {
        let mut cache = Cache::<DataTest>::new(0);
        let key = "key".to_string();
        let value = DataTest::default();
        cache.create(key.clone());
        cache.update(&key, value.clone());
        sleep(Duration::from_millis(1));

        assert_eq!(cache.get(&key).is_expired(), true);
    }

    #[test]
    fn value_unwrap_or() {
        let mut cache = Cache::<DataTest>::new(5);
        let key = "key".to_string();
        let value = DataTest::default();
        cache.create(key.clone());

        assert_eq!(cache.get(&key).unwrap_or(value.clone()), value);
    }

    #[test]
    fn value_remove() {
        let mut cache = Cache::<DataTest>::new(1000);
        let key = "key".to_string();
        let value = DataTest::default();
        cache.create(key.clone());
        cache.update(&key, value.clone());
        cache.remove(&key);

        assert_eq!(cache.get(&key).is_undefined(), true);
    }

    #[test]
    fn value_clean() {
        let mut cache = Cache::<DataTest>::new(1000);
        let key = "key".to_string();
        let value = DataTest::default();
        cache.create(key.clone());
        cache.update(&key, value.clone());
        cache.clean(&key);

        assert_eq!(cache.get(&key).is_empty(), true);
    }
}
