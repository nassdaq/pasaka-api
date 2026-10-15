//! The Swahili Bible, indexed at compile time.
//!
//! `build.rs` turns `src/data/swahili_bible.json` into one string of verse
//! text plus offset tables (`bible_index.rs` in `OUT_DIR`). A book is a range
//! of chapters, a chapter a range of verses, a verse a byte range of the text,
//! so a lookup is index arithmetic on static data and nothing is parsed while
//! running. The HTTP server preloads `books()` once; the function (`function.rs`)
//! runs as a fresh WebAssembly instance per call and uses the lookups directly.

use crate::models::bible::{Book, Chapter, SearchResult, Verse};

mod index {
    include!(concat!(env!("OUT_DIR"), "/bible_index.rs"));
}
use index::*;

#[derive(Clone, Copy)]
pub struct BookRef(usize);

#[derive(Clone, Copy)]
pub struct ChapterRef(usize);

#[derive(Clone, Copy)]
pub struct VerseRef(usize);

/// Every book, in Bible order.
pub fn books() -> impl Iterator<Item = BookRef> {
    (0..BOOK_NUMBERS.len()).map(BookRef)
}

/// A book by its number, as written in the request ("1", "66").
pub fn find_book(number: &str) -> Option<BookRef> {
    let wanted = parse(number)?;
    BOOK_NUMBERS.iter().position(|&n| n == wanted).map(BookRef)
}

impl BookRef {
    pub fn number(self) -> u16 {
        BOOK_NUMBERS[self.0]
    }

    pub fn name(self) -> &'static str {
        BOOK_NAMES[self.0]
    }

    pub fn chapters(self) -> impl Iterator<Item = ChapterRef> {
        range(&BOOK_CHAPTER_START, self.0).map(ChapterRef)
    }

    pub fn find_chapter(self, number: &str) -> Option<ChapterRef> {
        let wanted = parse(number)?;
        self.chapters().find(|c| c.number() == wanted)
    }

    pub fn to_book(self) -> Book {
        Book {
            id: self.number().to_string(),
            name: self.name().to_string(),
            chapters: self.chapters().map(ChapterRef::to_chapter).collect(),
        }
    }
}

impl ChapterRef {
    pub fn number(self) -> u16 {
        CHAPTER_NUMBERS[self.0]
    }

    pub fn verses(self) -> impl Iterator<Item = VerseRef> {
        range(&CHAPTER_VERSE_START, self.0).map(VerseRef)
    }

    pub fn find_verse(self, number: &str) -> Option<VerseRef> {
        let wanted = parse(number)?;
        self.verses().find(|v| v.number() == wanted)
    }

    pub fn to_chapter(self) -> Chapter {
        Chapter {
            chapter: self.number().to_string(),
            verses: self.verses().map(VerseRef::to_verse).collect(),
        }
    }
}

impl VerseRef {
    pub fn number(self) -> u16 {
        VERSE_NUMBERS[self.0]
    }

    pub fn text(self) -> &'static str {
        let (start, end) = range_bounds(&VERSE_TEXT_START, self.0);
        &TEXT[start..end]
    }

    pub fn to_verse(self) -> Verse {
        Verse {
            verse: self.number().to_string(),
            text: self.text().to_string(),
        }
    }
}

/// Every verse whose text contains `query`, ignoring case, in Bible order.
/// An empty query matches nothing.
pub fn search(query: &str) -> Vec<SearchResult> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    for book in books() {
        for chapter in book.chapters() {
            for verse in chapter.verses() {
                if contains_ignoring_case(verse.text(), &needle) {
                    results.push(SearchResult {
                        book: book.name().to_string(),
                        chapter: chapter.number().to_string(),
                        verse: verse.number().to_string(),
                        text: verse.text().to_string(),
                    });
                }
            }
        }
    }
    results
}

/// `text.to_lowercase().contains(needle)` without allocating for the ASCII
/// text this Bible is made of. `needle` is already lower-case.
fn contains_ignoring_case(text: &str, needle: &str) -> bool {
    if !text.is_ascii() {
        return text.to_lowercase().contains(needle);
    }
    if !needle.is_ascii() {
        return false; // lower-casing ASCII text never produces a non-ASCII char
    }
    let (text, needle) = (text.as_bytes(), needle.as_bytes());
    text.len() >= needle.len() && text.windows(needle.len()).any(|w| w.eq_ignore_ascii_case(needle))
}

fn parse(number: &str) -> Option<u16> {
    number.trim().parse().ok()
}

fn range(starts: &[u32], i: usize) -> std::ops::Range<usize> {
    let (start, end) = range_bounds(starts, i);
    start..end
}

fn range_bounds(starts: &[u32], i: usize) -> (usize, usize) {
    (starts[i] as usize, starts[i + 1] as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_whole_bible_is_there() {
        assert_eq!(books().count(), 66);
        assert_eq!(books().flat_map(BookRef::chapters).count(), 1189);
        assert_eq!(books().flat_map(BookRef::chapters).flat_map(ChapterRef::verses).count(), 31102);
    }

    #[test]
    fn looks_up_by_number() {
        let genesis = find_book("1").unwrap();
        assert_eq!(genesis.name(), "Mwanzo");
        assert_eq!(genesis.chapters().count(), 50);
        let first = genesis.find_chapter("1").unwrap().find_verse("1").unwrap();
        assert_eq!(first.text(), "Hapo mwanzo Mungu aliziumba mbingu na nchi. ");

        let revelation = find_book("66").unwrap().to_book();
        assert_eq!(revelation.chapters.len(), 22);
        assert_eq!(revelation.chapters[0].verses[0].verse, "1");
        assert!(revelation.chapters[0].verses[0].text.starts_with("Ufunuo wa Yesu Kristo"));

        assert!(find_book("67").is_none());
        assert!(find_book("one").is_none());
        assert!(genesis.find_chapter("51").is_none());
        assert!(genesis.find_chapter("1").unwrap().find_verse("32").is_none());
    }

    #[test]
    fn searches_ignoring_case() {
        let hits = search("HAPO MWANZO");
        assert_eq!(hits[0].book, "Mwanzo");
        assert_eq!(hits[0].chapter, "1");
        assert_eq!(hits[0].verse, "1");
        assert!(hits.iter().all(|h| h.text.to_lowercase().contains("hapo mwanzo")));
        assert!(search("").is_empty());
        assert!(search("   ").is_empty());
        assert!(search("xyzzy-not-in-the-bible").is_empty());
    }

    #[test]
    fn search_matches_the_allocating_definition() {
        for needle in ["mungu", "upendo", "yesu kristo", "A", "ß", "nchi."] {
            let lower = needle.to_lowercase();
            for verse in books().flat_map(BookRef::chapters).flat_map(ChapterRef::verses).take(3000) {
                assert_eq!(
                    contains_ignoring_case(verse.text(), &lower),
                    verse.text().to_lowercase().contains(&lower),
                    "{needle:?} in {:?}",
                    verse.text()
                );
            }
        }
    }
}
