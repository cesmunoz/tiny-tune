use super::{ArtworkCache, ArtworkEviction};

#[test]
fn evicts_the_oldest_entry_to_stay_inside_the_budget() {
    let mut cache = ArtworkCache::new(100);

    assert!(cache.insert("a".into(), "a", 60, None).stored);
    assert_eq!(
        cache.insert("b".into(), "b", 60, None).evicted,
        vec![ArtworkEviction { bytes: 60 }]
    );
    assert!(!cache.contains("a"));
    assert!(cache.contains("b"));
    assert_eq!(cache.used_bytes(), 60);

    cache.touch("b");
    let inserted = cache.insert("c".into(), "c", 50, None);
    assert!(inserted.stored);
    assert_eq!(inserted.evicted, vec![ArtworkEviction { bytes: 60 }]);
    assert!(!cache.contains("b"));
    assert_eq!(cache.peek("c"), Some(&"c"));
    assert!(cache.used_bytes() <= 100);
}

#[test]
fn keeps_the_protected_entry_and_refuses_an_oversize_insert() {
    let mut cache = ArtworkCache::new(100);
    assert!(
        cache
            .insert("current".into(), "cover", 80, Some("current"))
            .stored
    );

    let rejected = cache.insert("other".into(), "other", 80, Some("current"));
    assert!(!rejected.stored);
    assert_eq!(rejected.evicted, []);
    assert_eq!(cache.peek("current"), Some(&"cover"));
    assert_eq!(cache.used_bytes(), 80);

    let oversize = cache.insert("huge".into(), "huge", 101, Some("current"));
    assert!(!oversize.stored);
    assert_eq!(cache.used_bytes(), 80);
    assert_eq!(cache.peek("current"), Some(&"cover"));

    cache.insert("extra".into(), "extra", 20, Some("current"));
    let rejected = cache.insert("another".into(), "another", 50, Some("current"));
    assert!(!rejected.stored);
    assert_eq!(rejected.evicted, []);
    assert_eq!(cache.peek("current"), Some(&"cover"));
    assert_eq!(cache.peek("extra"), Some(&"extra"));
    assert_eq!(cache.used_bytes(), 100);
}

#[test]
fn replacing_a_key_does_not_count_its_bytes_twice() {
    let mut cache = ArtworkCache::new(100);
    cache.insert("a".into(), "first", 40, None);
    let replaced = cache.insert("a".into(), "second", 70, None);

    assert!(replaced.stored);
    assert_eq!(replaced.evicted, []);
    assert_eq!(cache.peek("a"), Some(&"second"));
    assert_eq!(cache.used_bytes(), 70);
}

#[test]
fn clear_drops_retained_bytes() {
    let mut cache = ArtworkCache::new(100);
    cache.insert("a".into(), "a", 40, None);
    cache.clear();

    assert_eq!(cache.used_bytes(), 0);
    assert!(!cache.contains("a"));
}
