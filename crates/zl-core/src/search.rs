//! Typo-tolerant title search: normalised words, trigrams and a bounded edit distance.

use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;

/// The comparable form of a text: no accents, lowercase, symbols as spaces, `&` as "and".
pub fn form(s: &str) -> String {
    if s.is_ascii() {
        form_ascii(s)
    } else {
        form_unicode(s)
    }
}

/// [`form`] of an ASCII text, in one pass: it has no accents to drop and nothing to decompose.
fn form_ascii(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    let mut gap = false;
    for b in s.bytes() {
        let and = b == b'&';
        if !and && !b.is_ascii_alphanumeric() {
            gap = true;
            continue;
        }
        if (gap || and) && !out.is_empty() {
            out.push(' ');
        }
        if and {
            out.push_str("and");
        } else {
            out.push(b.to_ascii_lowercase() as char);
        }
        gap = and;
    }
    out
}

/// [`form`] of any text: compatibility decomposition, combining marks dropped.
fn form_unicode(s: &str) -> String {
    let replaced = s.replace('&', " and ");
    let mut out = String::new();
    for c in replaced.nfkd() {
        if unicode_normalization::char::is_combining_mark(c) {
            continue;
        }
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The words of [`form`].
pub fn words(s: &str) -> Vec<String> {
    form(s)
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect()
}

/// The trigrams of every word of `s`, each word padded with spaces.
pub fn trigrams(s: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for w in words(s) {
        let p = format!("  {w} ");
        let chars: Vec<char> = p.chars().collect();
        if chars.len() >= 3 {
            for i in 0..=(chars.len() - 3) {
                out.insert(chars[i..i + 3].iter().collect());
            }
        }
    }
    out
}

/// The trigrams of `trigrams`, each packed into one `u64` (21 bits per char), sorted and unique.
pub fn trigram_codes(s: &str) -> Vec<u64> {
    trigram_codes_of_form(&form(s))
}

/// [`trigram_codes`] of a text that is already in its [`form`].
pub fn trigram_codes_of_form(form: &str) -> Vec<u64> {
    let mut out = Vec::new();
    let mut padded: Vec<char> = Vec::new();
    for w in form.split(' ').filter(|w| !w.is_empty()) {
        padded.clear();
        padded.extend([' ', ' ']);
        padded.extend(w.chars());
        padded.push(' ');
        for t in padded.windows(3) {
            out.push(((t[0] as u64) << 42) | ((t[1] as u64) << 21) | (t[2] as u64));
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Words longer than this (in characters) take the heap path of the distance.
const STACK: usize = 40;

/// The characters of `s` in `buf`, or None when they do not fit.
fn chars_in(s: &str, buf: &mut [char; STACK]) -> Option<usize> {
    let mut n = 0;
    for c in s.chars() {
        *buf.get_mut(n)? = c;
        n += 1;
    }
    Some(n)
}

/// Errors of query word `q` against title word `w`: a prefix counts as exact.
pub fn word_distance(q: &str, w: &str, max: usize) -> usize {
    let mut qb = ['\0'; STACK];
    match chars_in(q, &mut qb) {
        Some(n) => word_distance_chars(&qb[..n], w, max),
        None => {
            let qv: Vec<char> = q.chars().collect();
            word_distance_chars(&qv, w, max)
        }
    }
}

/// [`word_distance`] with the query word already split into characters.
pub fn word_distance_chars(q: &[char], w: &str, max: usize) -> usize {
    let mut wb = ['\0'; STACK];
    match chars_in(w, &mut wb) {
        Some(n) => word_distance_in(q, &wb[..n], max),
        None => {
            let wv: Vec<char> = w.chars().collect();
            word_distance_in(q, &wv, max)
        }
    }
}

/// Distance of query word `q` to word `wc`.
fn word_distance_in(q: &[char], wc: &[char], max: usize) -> usize {
    let q_len = q.len();
    let starts = wc.len() >= q_len && wc[..q_len] == *q;
    let mut d = if starts {
        0
    } else {
        distance_chars(q, wc, max)
    };
    if d > 0 && q_len > 3 && wc.len() > q_len {
        let d_prefix = distance_chars(q, &wc[..q_len], max);
        if d_prefix < d {
            d = d_prefix;
        }
    }
    d
}

/// Edit distance between `a` and `b`, stopping early and returning `max + 1` once exceeded.
pub fn distance(a: &str, b: &str, max: usize) -> usize {
    let (mut ab, mut bb) = (['\0'; STACK], ['\0'; STACK]);
    match (chars_in(a, &mut ab), chars_in(b, &mut bb)) {
        (Some(na), Some(nb)) => distance_chars(&ab[..na], &bb[..nb], max),
        _ => {
            let av: Vec<char> = a.chars().collect();
            let bv: Vec<char> = b.chars().collect();
            distance_chars(&av, &bv, max)
        }
    }
}

/// Damerau-Levenshtein distance (with adjacent transpositions).
pub fn distance_chars(a: &[char], b: &[char], max: usize) -> usize {
    if a.len().abs_diff(b.len()) > max {
        return max + 1;
    }
    let width = b.len() + 1;
    let mut stack = [0usize; 3 * (STACK + 1)];
    let mut heap: Vec<usize>;
    let rows: &mut [usize] = if width <= STACK + 1 {
        &mut stack[..3 * width]
    } else {
        heap = vec![0usize; 3 * width];
        &mut heap
    };
    let (mut two, mut prev, mut cur) = (0, width, 2 * width);
    for (j, cell) in rows[prev..prev + width].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        rows[cur] = i;
        let mut best = usize::MAX;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (rows[prev + j] + 1)
                .min(rows[cur + j - 1] + 1)
                .min(rows[prev + j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(rows[two + j - 2] + 1);
            }
            rows[cur + j] = v;
            best = best.min(v);
        }
        if best > max {
            return max + 1;
        }
        (two, prev, cur) = (prev, cur, two);
    }
    rows[prev + b.len()].min(max + 1)
}

/// The typos allowed in a query word of this length.
pub fn allowed(w: &str) -> usize {
    let len = w.chars().count();
    if len <= 2 {
        0
    } else if len <= 5 {
        1
    } else {
        2
    }
}

/// An index over a list of titles that finds them in spite of typos.
pub struct FuzzySearch {
    /// The words of every title.
    words: Vec<Vec<String>>,
    /// The trigrams of every title.
    grams: Vec<HashSet<String>>,
    /// For each trigram, the titles that hold it.
    index: HashMap<String, Vec<usize>>,
}

/// Candidate match with score metrics.
struct CandidateHit {
    /// Title index.
    i: usize,
    /// Sum of edit errors across query words.
    errors: usize,
    /// Dice similarity coefficient.
    dice: f64,
}

impl FuzzySearch {
    /// Builds the index of `titles`.
    pub fn new(titles: &[String]) -> Self {
        let words: Vec<Vec<String>> = titles.iter().map(|t| words(t)).collect();
        let grams: Vec<HashSet<String>> = titles.iter().map(|t| trigrams(t)).collect();
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, g_set) in grams.iter().enumerate() {
            for g in g_set {
                index.entry(g.clone()).or_default().push(i);
            }
        }

        Self {
            words,
            grams,
            index,
        }
    }

    /// Evaluates candidate indices against query words.
    fn rank_candidates(
        &self,
        q_words: &[String],
        q_grams: &HashSet<String>,
        shared: HashMap<usize, usize>,
    ) -> Vec<CandidateHit> {
        let mut hits = Vec::new();
        'outer: for (&i, &n) in &shared {
            let dice = (2.0 * n as f64) / (q_grams.len() + self.grams[i].len()) as f64;
            let mut errors = 0;
            for q in q_words {
                let max = allowed(q);
                let mut best = usize::MAX;
                for w in &self.words[i] {
                    let d = word_distance(q, w, max);
                    if d < best {
                        best = d;
                    }
                }
                if best > max {
                    continue 'outer;
                }
                errors += best;
            }
            hits.push(CandidateHit { i, errors, dice });
        }
        hits
    }

    /// Indices of the titles that match `query`, best first, at most `limit`.
    pub fn search(&self, query: &str, limit: usize) -> Vec<usize> {
        let q_words = words(query);
        if q_words.is_empty() {
            return Vec::new();
        }
        let q_grams = trigrams(query);
        let mut shared: HashMap<usize, usize> = HashMap::new();
        for t in &q_grams {
            if let Some(list) = self.index.get(t) {
                for &i in list {
                    *shared.entry(i).or_default() += 1;
                }
            }
        }
        let mut hits = self.rank_candidates(&q_words, &q_grams, shared);
        hits.sort_by(|a, b| {
            a.errors.cmp(&b.errors).then_with(|| {
                b.dice
                    .partial_cmp(&a.dice)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        hits.into_iter().take(limit).map(|h| h.i).collect()
    }
}
