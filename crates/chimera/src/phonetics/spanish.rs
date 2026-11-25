//! Spanish text → phoneme segments, in a neutral Latin American accent. Spanish spelling is close
//! to phonemic, so letters map to sounds by rule and no dictionary is needed (Quilis 1993;
//! Hualde 2005, "The Sounds of Spanish"):
//! - seseo: c before e/i, z and s are all /s/; yeísmo: ll and consonantal y are /ʝ/; j, and g before
//!   e/i, are /x/; h is silent; qu and gu before e/i are /k g/, gü is /gw/; ñ is /ɲ/; v is /b/.
//! - r is a trill [r] at the start of a word, after n, l and s, and written rr; a tap [ɾ] elsewhere.
//! - x is /ks/, /s/ at the start of a word, and /x/ in México and the names written like it.
//! - b d g are the approximants [β ð ɣ] except after a pause or a nasal (and d after l too), across
//!   word boundaries inside a phrase; a nasal takes the place of the consonant after it
//!   ([m] before p b m, [ɱ] before f, [ŋ] before k g x, [ɲ] before tʃ ʝ); s is [z] before a voiced
//!   consonant; p t k are unaspirated.
//! - i and u next to another vowel are the glides [j w] unless written with an accent (hiatus);
//!   a e o next to each other are two syllables.
//!
//! Stress follows the written accent; without one, a word ending in a vowel, n or s stresses the
//! penult and any other the last syllable. Unstressed function words (articles, clitic pronouns,
//! one-syllable prepositions and conjunctions) take none, and an adverb in -mente keeps its stem's
//! stress as a secondary one. Unstressed vowels don't reduce, and the rhythm is syllable-timed:
//! the duration rules (Klatt's, for English) shorten unstressed vowels far more than Spanish does,
//! so they are lengthened back (`UNSTRESSED_LENGTH`).
//!
//! The five vowels are the calibrated targets nearest Spanish speakers' formants (Quilis & Esgueva
//! 1983, men; Martínez Celdrán 1995): /a/ is the central [ɑ] of the table (F1/F2 731/1337 Hz, men's
//! /a/ about 700/1300), /u/ the back [u̠] (649 Hz F2; English [u]'s 997 is far front of Spanish
//! /u/'s 750-800); /i e o/ are the table's own.

use super::english::{Phrase, is_phrase_punct, split_keeping};
use super::mandarin::MandarinLexicon;
use super::syllables::{FrontEnd, Lang, lyrics_in};
use super::{Intonation, Segment, Syllable};

/// Unstressed vowels last this many times what the duration rules give them: Spanish stressed
/// vowels are 1.2-1.5 times the unstressed (Marín Gálvez 1995), where Klatt's rules make them about
/// twice, and Spanish's vowel nPVI is about half English's (Grabe & Low 2002: 29.7 against 57.2).
const UNSTRESSED_LENGTH: f64 = 1.25;
/// Klatt's /ɑ/ is the long vowel of "father" (240 ms inherent); Spanish /a/ is barely longer than
/// /e o/ (Navarro Tomás 1918), so it is brought to about /e/'s length.
const A_LENGTH: f64 = 0.8;

fn is_vowel_letter(c: char) -> bool {
    "aeiouáéíóúü".contains(c)
}

fn is_accented(c: char) -> bool {
    "áéíóú".contains(c)
}

/// e and i, after which c and g are soft
fn is_front_letter(c: char) -> bool {
    "eiéí".contains(c)
}

fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || "áéíóúüñÁÉÍÓÚÜÑ".contains(c)
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Words whose x is the old [x] (México and its family, and names written the old way).
const X_AS_J: &[&str] = &["méxico", "mexico", "mexicano", "mexicana", "mexicanos", "mexicanas", "oaxaca", "texas", "texano", "texana", "xavier", "ximena", "xalapa", "mexiquense"];

/// Unstressed function words: articles, clitic pronouns, possessives before a noun, one-syllable
/// prepositions and conjunctions, and the relatives and conjunctions written without an accent.
#[rustfmt::skip]
const UNSTRESSED: &[&str] = &[
    "el", "la", "los", "las", "lo", "le", "les", "me", "te", "se", "nos", "os",
    "mi", "mis", "tu", "tus", "su", "sus",
    "a", "al", "de", "del", "en", "con", "por", "sin", "para", "tras",
    "y", "e", "o", "u", "ni", "que", "si", "pero", "mas", "como", "cuando", "donde", "porque", "pues",
];

/// One sound of a word: its IPA, the letter its spelling starts at, and for a vowel, whether it is
/// written with an accent.
#[derive(Clone, Copy, Debug)]
struct Phone {
    ipa: &'static str,
    letter: usize,
    vowel: bool,
    accent: bool,
}

impl Phone {
    fn c(ipa: &'static str, letter: usize) -> Phone {
        Phone { ipa, letter, vowel: false, accent: false }
    }
}

fn vowel_ipa(c: char) -> &'static str {
    match c {
        'a' | 'á' => "ɑ",
        'e' | 'é' => "e",
        'i' | 'í' | 'y' => "i",
        'o' | 'ó' => "o",
        _ => "u̠", // u ú ü
    }
}

/// Letters (lower case) → phones, glides not yet decided (i and u are vowels here).
fn letters_to_phones(w: &[char], x_as_j: bool) -> Vec<Phone> {
    let mut out: Vec<Phone> = Vec::with_capacity(w.len() + 2);
    let at = |i: usize| w.get(i).copied();
    let front_after = |i: usize| at(i).is_some_and(is_front_letter);
    // a silent h's letter: the next sound's spelling starts there
    let mut pending: Option<usize> = None;
    let mut i = 0;
    while i < w.len() {
        let c = w[i];
        let start = pending.take().unwrap_or(i);
        let push = |out: &mut Vec<Phone>, ipa: &'static str| out.push(Phone::c(ipa, start));
        match c {
            'h' => {
                // hue-, hui- start with [w] ("hueso", "huevo"); otherwise h is silent
                if at(i + 1) == Some('u') && at(i + 2).is_some_and(|n| is_vowel_letter(n) && n != 'u') {
                    push(&mut out, "w");
                    i += 2;
                    continue;
                }
                // hie- at the start of a word is consonantal ("hielo", "hierba": [ʝ])
                if i == 0 && at(i + 1) == Some('i') && at(i + 2).is_some_and(|n| matches!(n, 'e' | 'é' | 'a' | 'o')) {
                    push(&mut out, "ʝ");
                    i += 2;
                    continue;
                }
                pending = Some(start);
            }
            'c' if at(i + 1) == Some('h') => {
                push(&mut out, "tʃ");
                i += 1;
            }
            'c' if front_after(i + 1) => push(&mut out, "s"),
            'c' | 'k' => push(&mut out, "k"),
            'q' => {
                push(&mut out, "k");
                if at(i + 1) == Some('u') {
                    i += 1;
                }
            }
            'g' if front_after(i + 1) => push(&mut out, "x"),
            'g' => {
                push(&mut out, "g");
                // gue, gui: the u is silent (gü keeps it, as [w])
                if at(i + 1) == Some('u') && front_after(i + 2) {
                    i += 1;
                }
            }
            'j' => push(&mut out, "x"),
            'l' if at(i + 1) == Some('l') => {
                push(&mut out, "ʝ");
                i += 1;
            }
            'l' => push(&mut out, "l"),
            'r' if at(i + 1) == Some('r') => {
                push(&mut out, "r");
                i += 1;
            }
            'r' => {
                let trill = i == 0 || i.checked_sub(1).map(|p| w[p]).is_some_and(|p| "nls".contains(p));
                push(&mut out, if trill { "r" } else { "ɾ" });
            }
            'ñ' => push(&mut out, "ɲ"),
            'b' | 'v' => push(&mut out, "b"),
            'z' | 's' => push(&mut out, "s"),
            'x' if x_as_j => push(&mut out, "x"),
            'x' if i == 0 => push(&mut out, "s"),
            'x' => {
                push(&mut out, "k");
                out.push(Phone::c("s", start));
            }
            'y' => {
                // a vowel at the end of a word after a vowel ("hoy", "muy", "rey") and before a
                // consonant; a consonant before a vowel ("yo", "mayo")
                if at(i + 1).is_some_and(is_vowel_letter) {
                    push(&mut out, "ʝ");
                } else {
                    out.push(Phone { ipa: "i", letter: start, vowel: true, accent: false });
                }
            }
            'w' => push(&mut out, "w"),
            'd' => push(&mut out, "d"),
            'f' => push(&mut out, "f"),
            'm' => push(&mut out, "m"),
            'n' => push(&mut out, "n"),
            'p' => {
                // ps- and pt- at the start of a word drop the p ("psicología", "pterodáctilo")
                if !(i == 0 && matches!(at(i + 1), Some('s' | 't'))) {
                    push(&mut out, "p");
                } else {
                    pending = Some(start);
                }
            }
            't' => push(&mut out, "t"),
            c if is_vowel_letter(c) => out.push(Phone { ipa: vowel_ipa(c), letter: start, vowel: true, accent: is_accented(c) }),
            _ => {}
        }
        i += 1;
    }
    out
}

/// i and u (unaccented, and y as a vowel, and ü) next to another vowel become the glides [j w]: before
/// a vowel they open its syllable ("bien", "cuando"; of "iu"/"ui" the first glides: "ciudad",
/// "cuidado"), after one they close it ("aire", "auto", "hoy"). Two strong vowels stay two syllables.
fn glides(ph: &mut [Phone], w: &[char]) {
    let weak = |p: &Phone| p.vowel && !p.accent && matches!(p.ipa, "i" | "u̠");
    let n = ph.len();
    for k in 0..n {
        if !weak(&ph[k]) {
            continue;
        }
        let next_vowel = k + 1 < n && ph[k + 1].vowel;
        let prev_nucleus = k > 0 && ph[k - 1].vowel;
        let dieresis = w.get(ph[k].letter) == Some(&'ü');
        if next_vowel || prev_nucleus || dieresis {
            ph[k].vowel = false;
            ph[k].ipa = if ph[k].ipa == "i" { "j" } else { "w" };
        }
    }
}

/// A consonant cluster (IPA) that can start a Spanish syllable: one consonant; an obstruent and a
/// liquid (pl pɾ bl bɾ tɾ dɾ kl kɾ gl gɾ fl fɾ, and Mexican tl); either followed by a glide.
fn is_onset(c: &[&str]) -> bool {
    let liquid_ok = |a: &str, b: &str| matches!(a, "p" | "b" | "k" | "g" | "f") && matches!(b, "l" | "ɾ") || matches!((a, b), ("t", "ɾ") | ("d", "ɾ") | ("t", "l"));
    let glide = |x: &str| matches!(x, "j" | "w");
    match c {
        [_] => true,
        [a, b] => liquid_ok(a, b) || (glide(b) && !glide(a)),
        [a, b, g] => liquid_ok(a, b) && glide(g),
        _ => false,
    }
}

/// Phones → syllables: the index of each syllable's first phone (nucleus = its vowel).
fn syllable_starts(ph: &[Phone]) -> Vec<usize> {
    let vi: Vec<usize> = ph.iter().enumerate().filter(|(_, p)| p.vowel).map(|(i, _)| i).collect();
    let mut starts = Vec::with_capacity(vi.len());
    for (n, &v) in vi.iter().enumerate() {
        if n == 0 {
            starts.push(0);
            continue;
        }
        let prev = vi[n - 1];
        let between: Vec<&str> = ph[prev + 1..v].iter().map(|p| p.ipa).collect();
        let mut k = 0;
        for len in (1..=between.len()).rev() {
            if is_onset(&between[between.len() - len..]) {
                k = len;
                break;
            }
        }
        starts.push(v - k);
    }
    starts
}

/// The stressed syllable of a word with `n` syllables, by the written accent or the default rule.
fn stressed_syllable(ph: &[Phone], starts: &[usize], last_letter: char) -> usize {
    let n = starts.len();
    let syl_of = |i: usize| starts.iter().rposition(|&s| s <= i).unwrap_or(0);
    if let Some(i) = ph.iter().position(|p| p.vowel && p.accent) {
        return syl_of(i);
    }
    if n >= 2 && (is_vowel_letter(last_letter) || last_letter == 'n' || last_letter == 's') {
        n - 2
    } else {
        n - 1
    }
}

/// A word read: its letters (lower case, and as typed), its sounds, the start of each syllable and
/// each syllable's stress.
struct Parsed {
    typed: Vec<char>,
    phones: Vec<Phone>,
    starts: Vec<usize>,
    stress: Vec<u8>,
}

fn parse(typed: &str) -> Parsed {
    let kept: Vec<char> = typed.chars().filter(|c| is_letter(*c)).collect();
    let letters: Vec<char> = kept.iter().map(|c| lower(*c)).collect();
    let word: String = letters.iter().collect();
    let x_as_j = X_AS_J.contains(&word.as_str());
    // -mente: the stem keeps its own stress (secondary), "men" takes the primary
    let mente = word.len() > 6 && word.ends_with("mente") && letters[..letters.len() - 5].iter().any(|c| is_vowel_letter(*c));
    let mut phones = letters_to_phones(&letters, x_as_j);
    glides(&mut phones, &letters);
    let starts = syllable_starts(&phones);
    let n = starts.len();
    let mut stress = vec![0u8; n];
    if n > 0 && !UNSTRESSED.contains(&word.as_str()) {
        if mente && n >= 3 {
            let stem_letters = &letters[..letters.len() - 5];
            let stem_phones: Vec<Phone> = phones.iter().copied().filter(|p| p.letter < stem_letters.len()).collect();
            let stem_starts = syllable_starts(&stem_phones);
            if !stem_starts.is_empty() {
                stress[stressed_syllable(&stem_phones, &stem_starts, *stem_letters.last().unwrap())] = 2;
            }
            stress[n - 2] = 1;
        } else {
            stress[stressed_syllable(&phones, &starts, *letters.last().unwrap())] = 1;
        }
    }
    Parsed { typed: kept, phones, starts, stress }
}

/// Context-dependent sounds of one phrase's phones, `boundary` true after each word's last phone:
/// spirantisation of b d g, nasal place, voiced s.
fn allophones(ipa: &mut [&'static str], phrase_initial: bool) {
    let n = ipa.len();
    let nasal = |x: &str| matches!(x, "m" | "n" | "ɲ" | "ŋ" | "ɱ");
    // underlying forms first: assimilation reads the stop after a nasal, which then stays a stop
    let under: Vec<&str> = ipa.to_vec();
    for k in 0..n {
        let prev = if k > 0 { Some(under[k - 1]) } else { None };
        ipa[k] = match under[k] {
            "b" | "d" | "g" if !(k == 0 && phrase_initial) => {
                let stop = prev.is_some_and(nasal) || (under[k] == "d" && prev == Some("l"));
                if stop {
                    under[k]
                } else {
                    match under[k] {
                        "b" => "β",
                        "d" => "ð",
                        _ => "ɣ",
                    }
                }
            }
            "n" => match under.get(k + 1).copied() {
                Some("p" | "b" | "m") => "m",
                Some("f") => "ɱ",
                Some("k" | "g" | "x") => "ŋ",
                Some("tʃ" | "ʝ" | "ɲ") => "ɲ",
                _ => "n",
            },
            "s" => match under.get(k + 1).copied() {
                Some("b" | "d" | "g" | "m" | "n" | "l" | "ʝ" | "r" | "ɲ") => "z",
                _ => "s",
            },
            x => x,
        };
    }
}

/// Segments for words (each word's phones and stresses), with the phrase's allophones.
fn segments(words: &[Parsed], phrase_initial: bool) -> Vec<Segment> {
    let mut ipa: Vec<&'static str> = words.iter().flat_map(|w| w.phones.iter().map(|p| p.ipa)).collect();
    allophones(&mut ipa, phrase_initial);
    let mut out = Vec::with_capacity(ipa.len());
    let mut k = 0;
    for w in words {
        for (i, p) in w.phones.iter().enumerate() {
            let syl = w.starts.iter().rposition(|&s| s <= i).unwrap_or(0);
            out.push(segment(ipa[k], p.vowel, w.stress.get(syl).copied().unwrap_or(0), i + 1 == w.phones.len()));
            k += 1;
        }
    }
    out
}

fn segment(ipa: &'static str, vowel: bool, stress: u8, word_end: bool) -> Segment {
    let length = if vowel {
        let k = if stress == 0 { UNSTRESSED_LENGTH } else { 1.0 } * if ipa == "ɑ" { A_LENGTH } else { 1.0 };
        (k != 1.0).then_some(k)
    } else {
        None
    };
    Segment {
        ipa: ipa.to_string(),
        stress: vowel.then_some(stress),
        // Spanish voiceless stops are unaspirated (VOT 10-30 ms: Lisker & Abramson 1964)
        aspirated: matches!(ipa, "p" | "t" | "k").then_some(false),
        length,
        word_end: word_end.then_some(true),
        ..Default::default()
    }
}

/// One word, as typed, to segments with stress (as said after a pause when `phrase_initial`).
pub fn word(typed: &str, phrase_initial: bool) -> Vec<Segment> {
    let p = parse(typed);
    let mut s = segments(std::slice::from_ref(&p), phrase_initial);
    if let Some(last) = s.last_mut() {
        last.word_end = None;
    }
    s
}

/// Whether a word ends in a sound after which b d g stay stops (a nasal, and l for d).
fn ends_in_nasal(typed: &str) -> bool {
    parse(typed).phones.last().is_some_and(|p| matches!(p.ipa, "m" | "n" | "ɲ"))
}

/// One word's sung syllables: spelling split where its sounds split ("ca-mi-no", "pe-rro",
/// "ciu-dad"). `after`: the word before it in the phrase, if any (b d g after it are approximants
/// unless it ends in a nasal).
fn sung_syllables(typed: &str, after: Option<&str>) -> Vec<Syllable> {
    let p = parse(typed);
    if p.starts.is_empty() {
        return Vec::new();
    }
    let mut ipa: Vec<&'static str> = p.phones.iter().map(|x| x.ipa).collect();
    match after {
        None => allophones(&mut ipa, true),
        Some(prev) => {
            // the word before's last sound decides this one's first
            let lead = if ends_in_nasal(prev) { "n" } else { "ɑ" };
            let mut with: Vec<&'static str> = std::iter::once(lead).chain(ipa.iter().copied()).collect();
            allophones(&mut with, false);
            ipa.copy_from_slice(&with[1..]);
        }
    }
    let n = p.starts.len();
    let letter_starts: Vec<usize> = (0..n).map(|k| if k == 0 { 0 } else { p.phones[p.starts[k]].letter }).collect();
    let mut out: Vec<Syllable> = (0..n)
        .map(|k| {
            let b = letter_starts.get(k + 1).copied().unwrap_or(p.typed.len());
            Syllable { text: p.typed[letter_starts[k]..b].iter().collect(), ..Default::default() }
        })
        .collect();
    for (i, ph) in p.phones.iter().enumerate() {
        let k = p.starts.iter().rposition(|&s| s <= i).unwrap_or(0);
        // a note sets its own lengths
        let seg = Segment { length: None, word_end: None, ..segment(ipa[i], ph.vowel, p.stress[k], false) };
        if ph.vowel {
            out[k].nucleus = seg;
        } else if out[k].nucleus.ipa.is_empty() {
            out[k].onset.push(seg);
        } else {
            out[k].coda.push(seg);
        }
    }
    out
}

/// The Spanish front end for the lyrics router ([`super::syllables::read_lyrics`]). Spanish is
/// written in Latin letters, so the router can't tell it from English by script: the player chooses
/// it ([`Lang::Es`]). Typed hyphens split a word your way, as in English.
pub struct Spanish;

impl FrontEnd for Spanish {
    fn starts(&self, c: char) -> bool {
        is_letter(c)
    }
    fn continues(&self, c: char) -> bool {
        is_letter(c) || c == '-'
    }
    fn word(&self, tok: &str) -> Vec<Syllable> {
        self.word_after(tok, None)
    }
    fn word_after(&self, tok: &str, prev: Option<&str>) -> Vec<Syllable> {
        let mut out = Vec::new();
        let mut before = prev.map(str::to_string);
        for part in tok.split('-').filter(|p| p.chars().any(is_letter)) {
            out.append(&mut sung_syllables(part, before.as_deref()));
            before = Some(part.to_string());
        }
        out
    }
}

/// Lyrics → sung syllables with Latin script read as Spanish ([`Lang::Es`]); other scripts are
/// routed as usual, and no dictionary is needed.
pub fn lyrics_to_syllables(text: &str) -> Vec<Syllable> {
    lyrics_in(text, &|_: &str, _: bool| Vec::<Segment>::new(), &MandarinLexicon::new(), Lang::Es)
}

// ---------------------------------------------------------------- phrases and intonation

/// Question words that start a wh-question, which falls like a statement (Spanish yes/no questions
/// rise at the end; Navarro Tomás 1944, Sosa 1999).
const WH: &[&str] = &["qué", "quién", "quiénes", "cómo", "dónde", "adónde", "cuándo", "cuál", "cuáles", "cuánto", "cuánta", "cuántos", "cuántas"];

/// Spanish text → phrases of segments. Phrase punctuation breaks phrases; a phrase ending in a
/// comma, semicolon or colon continues (a rise: list items, a clause before the main one), a
/// question rises unless it starts with a question word, and the rest fall. Numbers and common
/// abbreviations are read out.
pub fn to_phrases(text: &str) -> Vec<Phrase> {
    let normalized = expand_numbers(&expand_abbreviations(text));
    let mut phrases: Vec<Phrase> = Vec::new();
    let mut words: Vec<String> = Vec::new();
    // whether the sentence being read started with a question word
    let mut wh = false;
    let mut sentence_start = true;
    let starts_wh = |ws: &[String]| WH.contains(&ws[0].to_lowercase().as_str()) || (ws.len() > 1 && ws[0].eq_ignore_ascii_case("por") && ws[1].to_lowercase() == "qué");
    let words_of = |t: &str| t.split(|c: char| !is_letter(c)).filter(|w| !w.is_empty()).map(str::to_string).collect::<Vec<_>>();
    for part in split_keeping(&normalized, is_phrase_punct) {
        if !part.is_empty() && part.chars().all(is_phrase_punct) {
            let delim = part;
            let ws = std::mem::take(&mut words);
            if ws.is_empty() {
                continue;
            }
            if sentence_start {
                wh = starts_wh(&ws);
            }
            let ends = delim.contains(['.', '!', '?']);
            let intonation = if delim.contains('?') {
                if wh { Intonation::Statement } else { Intonation::Question }
            } else if ends {
                Intonation::Statement
            } else {
                Intonation::Continuation
            };
            let pause = if ends { 0.35 } else { 0.18 };
            push_phrase(&mut phrases, &ws, intonation, pause);
            sentence_start = ends;
        } else {
            // a question opened mid-sentence ("Juan, ¿dónde estás?") is a question word's or not by
            // its own first word
            if let Some(k) = part.rfind('¿') {
                let q = words_of(&part[k..]);
                if !q.is_empty() {
                    wh = starts_wh(&q);
                    sentence_start = false;
                }
            }
            words.extend(words_of(&part));
        }
    }
    if !words.is_empty() {
        push_phrase(&mut phrases, &words, Intonation::Statement, 0.3);
    }
    phrases
}

fn push_phrase(phrases: &mut Vec<Phrase>, words: &[String], intonation: Intonation, pause: f64) {
    let parsed: Vec<Parsed> = words.iter().map(|w| parse(w)).filter(|p| !p.phones.is_empty()).collect();
    if parsed.is_empty() {
        return;
    }
    let said = parsed.iter().map(|p| p.typed.iter().map(|c| lower(*c)).collect()).collect();
    phrases.push(Phrase { segments: segments(&parsed, true), intonation, pause, words: said });
}

// ---------------------------------------------------------------- abbreviations

/// Abbreviations read out (matched as whole words, with their dot).
#[rustfmt::skip]
const ABBREVIATIONS: &[(&str, &str)] = &[
    ("EE. UU.", "Estados Unidos"), ("EE.UU.", "Estados Unidos"), ("EEUU", "Estados Unidos"),
    ("p. ej.", "por ejemplo"), ("Srta.", "señorita"), ("Sras.", "señoras"), ("Sres.", "señores"),
    ("Sra.", "señora"), ("Sr.", "señor"), ("Dra.", "doctora"), ("Dr.", "doctor"), ("Uds.", "ustedes"),
    ("Ud.", "usted"), ("Vd.", "usted"), ("Lic.", "licenciado"), ("Ing.", "ingeniero"), ("Prof.", "profesor"),
    ("Profa.", "profesora"), ("Avda.", "avenida"), ("Av.", "avenida"), ("núm.", "número"), ("Núm.", "número"),
    ("Nº", "número"), ("pág.", "página"), ("etc.", "etcétera"), ("aprox.", "aproximadamente"),
    ("tel.", "teléfono"), ("depto.", "departamento"), ("Cía.", "compañía"), ("admin.", "administración"),
];

/// Units read out after a number ("5 km").
const UNITS: &[(&str, &str)] = &[("km", "kilómetros"), ("kg", "kilos"), ("cm", "centímetros"), ("mm", "milímetros"), ("ml", "mililitros"), ("min", "minutos"), ("h", "horas"), ("%", "por ciento")];

fn expand_abbreviations(t: &str) -> String {
    let mut s = t.replace('’', "'");
    for (abbr, full) in ABBREVIATIONS {
        let mut out = String::with_capacity(s.len());
        let mut rest = s.as_str();
        while let Some(k) = rest.find(abbr) {
            let before_ok = rest[..k].chars().next_back().is_none_or(|c| !is_letter(c));
            let after = &rest[k + abbr.len()..];
            let after_ok = abbr.ends_with('.') || after.chars().next().is_none_or(|c| !is_letter(c));
            out.push_str(&rest[..k]);
            if before_ok && after_ok {
                out.push_str(full);
            } else {
                out.push_str(abbr);
            }
            rest = after;
        }
        out.push_str(rest);
        s = out;
    }
    s
}

// ---------------------------------------------------------------- numbers

const UNITS_0_29: [&str; 30] = [
    "cero", "uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez", "once", "doce", "trece", "catorce", "quince", "dieciséis", "diecisiete", "dieciocho", "diecinueve", "veinte", "veintiuno", "veintidós", "veintitrés",
    "veinticuatro", "veinticinco", "veintiséis", "veintisiete", "veintiocho", "veintinueve",
];
const TENS: [&str; 10] = ["", "", "", "treinta", "cuarenta", "cincuenta", "sesenta", "setenta", "ochenta", "noventa"];
const HUNDREDS: [&str; 10] = ["", "ciento", "doscientos", "trescientos", "cuatrocientos", "quinientos", "seiscientos", "setecientos", "ochocientos", "novecientos"];
const ORDINALS: [&str; 11] = ["", "primero", "segundo", "tercero", "cuarto", "quinto", "sexto", "séptimo", "octavo", "noveno", "décimo"];

/// 0..999 as words; `apocope` reads a final "uno" as "un" (before "mil", "millones", a noun).
fn below_thousand(n: u64, apocope: bool, out: &mut Vec<String>) {
    let (h, r) = ((n / 100) as usize, (n % 100) as usize);
    if n == 100 {
        out.push("cien".into());
        return;
    }
    if h > 0 {
        out.push(HUNDREDS[h].into());
    }
    if r == 0 {
        return;
    }
    let mut words: Vec<String> = if r < 30 {
        vec![UNITS_0_29[r].into()]
    } else if r % 10 == 0 {
        vec![TENS[r / 10].into()]
    } else {
        vec![TENS[r / 10].into(), "y".into(), UNITS_0_29[r % 10].into()]
    };
    if apocope {
        let last = words.last_mut().unwrap();
        if last == "uno" {
            *last = "un".into();
        } else if last == "veintiuno" {
            *last = "veintiún".into();
        }
    }
    out.append(&mut words);
}

/// Digits → Spanish number words (digit by digit past the trillions).
fn number_words(digits: &str) -> String {
    let d = digits.trim_start_matches('0');
    if d.is_empty() {
        return "cero".into();
    }
    let Some(n) = d.parse::<u64>().ok().filter(|n| *n < 1_000_000_000_000_000) else {
        return digits.bytes().map(|b| UNITS_0_29[(b - b'0') as usize]).collect::<Vec<_>>().join(" ");
    };
    let mut out: Vec<String> = Vec::new();
    let (billions, millions, rest) = (n / 1_000_000_000_000, n / 1_000_000 % 1_000_000, n % 1_000_000);
    // long scale: a billón is a million millions, and a thousand millions are "mil millones"
    let thousands_of = |m: u64, out: &mut Vec<String>| {
        let (th, u) = (m / 1000, m % 1000);
        if th > 0 {
            if th > 1 {
                below_thousand(th, true, out);
            }
            out.push("mil".into());
        }
        below_thousand(u, true, out);
    };
    if billions > 0 {
        thousands_of(billions, &mut out);
        out.push(if billions == 1 { "billón" } else { "billones" }.into());
    }
    if millions > 0 {
        thousands_of(millions, &mut out);
        out.push(if millions == 1 { "millón" } else { "millones" }.into());
    }
    let (th, u) = (rest / 1000, rest % 1000);
    if th > 0 {
        if th > 1 {
            below_thousand(th, true, &mut out);
        }
        out.push("mil".into());
    }
    below_thousand(u, false, &mut out);
    out.join(" ")
}

/// Numbers in text → words. A dot before exactly three digits groups thousands ("1.000"), as does
/// a space; a comma between digits is the decimal comma ("3,5": "tres coma cinco") and any other
/// dot between digits the decimal point ("3.5": "tres punto cinco"). "10:30" is "diez y treinta",
/// "1º" "primero", and a unit after a number is read out ("5 km", "20 %").
fn expand_numbers(t: &str) -> String {
    let c: Vec<char> = t.chars().collect();
    let digit = |i: usize| c.get(i).is_some_and(char::is_ascii_digit);
    let mut out = String::with_capacity(t.len());
    let mut i = 0;
    while i < c.len() {
        if !digit(i) {
            out.push(c[i]);
            i += 1;
            continue;
        }
        let mut int = String::new();
        loop {
            while digit(i) {
                int.push(c[i]);
                i += 1;
            }
            if c.get(i) == Some(&'.') && digit(i + 1) && digit(i + 2) && digit(i + 3) && !digit(i + 4) {
                i += 1;
                continue;
            }
            break;
        }
        out.push(' ');
        // ordinals: 1º 2ª (up to 10)
        if let Some(&o) = c.get(i).filter(|o| matches!(**o, 'º' | 'ª' | '°')) {
            match int.parse::<usize>() {
                Ok(k @ 1..=10) => {
                    let w = ORDINALS[k];
                    out.push_str(&if o == 'ª' { format!("{}a", &w[..w.len() - 1]) } else { w.to_string() });
                }
                _ => out.push_str(&number_words(&int)),
            }
            out.push(' ');
            i += 1;
            continue;
        }
        out.push_str(&number_words(&int));
        if (c.get(i) == Some(&',') || c.get(i) == Some(&'.')) && digit(i + 1) {
            out.push_str(if c[i] == ',' { " coma" } else { " punto" });
            i += 1;
            let mut frac = String::new();
            while digit(i) {
                frac.push(c[i]);
                i += 1;
            }
            let zeros = frac.len() - frac.trim_start_matches('0').len();
            for _ in 0..zeros {
                out.push_str(" cero");
            }
            if zeros < frac.len() {
                out.push(' ');
                out.push_str(&number_words(&frac[zeros..]));
            }
        } else if c.get(i) == Some(&':') && digit(i + 1) && digit(i + 2) {
            // a time of day
            let mm: String = c[i + 1..i + 3].iter().collect();
            i += 3;
            if mm != "00" {
                out.push_str(" y ");
                out.push_str(&number_words(&mm));
            }
        }
        // a unit after the number
        let mut j = i;
        while c.get(j) == Some(&' ') {
            j += 1;
        }
        for (u, full) in UNITS {
            let uc: Vec<char> = u.chars().collect();
            if c.len() >= j + uc.len() && c[j..j + uc.len()] == uc[..] && c.get(j + uc.len()).is_none_or(|x| !is_letter(*x)) {
                out.push(' ');
                out.push_str(full);
                i = j + uc.len();
                break;
            }
        }
        out.push(' ');
    }
    out
}
