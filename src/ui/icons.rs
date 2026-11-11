//! Lucide icons (ISC, see `assets/icons/LICENSE`) as Rune icon bodies.
//!
//! Rune's icon renderer reads a subset of SVG: `path`, `rect`, `circle`,
//! `ellipse` and `line`, with stroke attributes on each element. Lucide's
//! polygons and polylines are written here as closed or open paths.

use rune::Icon;

macro_rules! lucide {
    ($name:ident, $id:literal, $($element:literal),+ $(,)?) => {
        pub static $name: Icon = Icon {
            name: concat!("tiny-tune-", $id),
            view: 24.0,
            body: concat!($($element),+),
        };
    };
}

lucide!(
    PLAY,
    "play",
    r#"<path d="M6 3L20 12L6 21Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
);
lucide!(
    PAUSE,
    "pause",
    r#"<rect width="4" height="16" x="6" y="4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    r#"<rect width="4" height="16" x="14" y="4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
);
lucide!(
    SKIP_BACK,
    "skip-back",
    r#"<path d="M19 20L9 12L19 4Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    r#"<line x1="5" x2="5" y1="19" y2="5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
);
lucide!(
    SKIP_FORWARD,
    "skip-forward",
    r#"<path d="M5 4L15 12L5 20Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    r#"<line x1="19" x2="19" y1="5" y2="19" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
);
lucide!(
    VOLUME_1,
    "volume-1",
    r#"<path d="M11 5L6 9L2 9L2 15L6 15L11 19Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    r#"<path d="M15.54 8.46a5 5 0 0 1 0 7.07" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
);
lucide!(
    VOLUME_2,
    "volume-2",
    r#"<path d="M11 5L6 9L2 9L2 15L6 15L11 19Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    r#"<path d="M15.54 8.46a5 5 0 0 1 0 7.07" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    r#"<path d="M19.07 4.93a10 10 0 0 1 0 14.14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
);
lucide!(
    LOG_OUT,
    "log-out",
    r#"<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    r#"<path d="M16 17L21 12L16 7" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    r#"<line x1="21" x2="9" y1="12" y2="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
);
