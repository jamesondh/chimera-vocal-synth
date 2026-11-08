//! `chimera tajweed score [dir]`: the Arabic reader's tajweed annotations against an expert
//! annotation of the whole Qurʾān, rule by rule.
//!
//! The reference is cpfair/quran-tajweed (riwāyat Ḥafṣ, CC BY 4.0: built from ReciteQuran.com and
//! the Dar al-Maarifah tajweed muṣḥaf), indexed into its copy of the Tanzil Uthmani text. Neither
//! file is in this repository (Tanzil's terms); `tools/fetch-tajweed.sh` puts both in
//! ~/datasets/quran-tajweed.
//!
//! Each āya is read alone, in pause at its end. A reference span is found when an annotation of
//! the same rule overlaps it (recall); an annotation is right when it overlaps a reference span of
//! its rule (precision).

use chimera::phonetics::arabic::{ArabicOptions, to_phrases};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Ayah {
    surah: usize,
    ayah: usize,
    annotations: Vec<Span>,
}

#[derive(Deserialize, Clone)]
struct Span {
    rule: String,
    start: usize,
    end: usize,
}

fn overlaps(a: &Span, b: &Span) -> bool {
    a.start < b.end && b.start < a.end
}

pub fn score(a: &[String]) -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let default = format!("{home}/datasets/quran-tajweed");
    let dir = a.first().map(String::as_str).unwrap_or(&default);
    let show: Option<&str> = a.iter().position(|x| x == "--show").and_then(|i| a.get(i + 1)).map(String::as_str);
    let text = std::fs::read_to_string(format!("{dir}/quran-uthmani.txt")).map_err(|e| format!("{dir}/quran-uthmani.txt: {e} (run tools/fetch-tajweed.sh)"))?;
    let gold: Vec<Ayah> = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/tajweed.json")).map_err(|e| format!("{dir}/tajweed.json: {e}"))?).map_err(|e| e.to_string())?;
    let verses: Vec<&str> = text.lines().filter(|l| !l.is_empty() && !l.starts_with('#')).map(|l| l.splitn(3, '|').nth(2).unwrap_or("")).collect();
    if verses.len() != gold.len() {
        return Err(format!("{} verses but {} annotated", verses.len(), gold.len()));
    }
    // rule → [gold, gold found, mine, mine right]
    let mut tally: BTreeMap<String, [usize; 4]> = BTreeMap::new();
    let mut shown = 0;
    let o = ArabicOptions::default();
    // the text prefixes each sūra's first āya (but al-Fātiḥa's and at-Tawba's) with the basmala,
    // its first four words, which the reference leaves bare
    for (v, g) in verses.iter().zip(&gold) {
        let skip = if g.ayah == 1 && g.surah != 1 && g.surah != 9 && v.starts_with("بِسْمِ") {
            v.char_indices().filter(|(_, c)| *c == ' ').nth(3).map_or(0, |(b, _)| v[..b].chars().count())
        } else {
            0
        };
        let mine: Vec<Span> = to_phrases(v, &o).iter().flat_map(|p| p.annotations.iter().map(|x| Span { rule: x.rule.to_string(), start: x.start, end: x.end })).filter(|x| x.start >= skip).collect();
        let chars: Vec<char> = v.chars().collect();
        let cut = |s: &Span| chars[s.start.min(chars.len())..s.end.min(chars.len())].iter().collect::<String>();
        for s in &g.annotations {
            let t = tally.entry(s.rule.clone()).or_default();
            t[0] += 1;
            let found = mine.iter().any(|m| m.rule == s.rule && overlaps(m, s));
            t[1] += found as usize;
            if !found && show == Some(s.rule.as_str()) && shown < 25 {
                shown += 1;
                let said: Vec<String> = mine.iter().filter(|m| overlaps(m, s)).map(|m| m.rule.clone()).collect();
                println!("  missed {} {}:{} [{}] in …{}… (said {:?})", s.rule, g.surah, g.ayah, cut(s), chars[s.start.saturating_sub(8)..(s.end + 8).min(chars.len())].iter().collect::<String>(), said);
            }
        }
        for m in &mine {
            let t = tally.entry(m.rule.clone()).or_default();
            t[2] += 1;
            let right = g.annotations.iter().any(|s| s.rule == m.rule && overlaps(m, s));
            t[3] += right as usize;
            if !right && show == Some(m.rule.as_str()) && shown < 50 {
                shown += 1;
                println!("  extra  {} {}:{} [{}] in …{}…", m.rule, g.surah, g.ayah, cut(m), chars[m.start.saturating_sub(8)..(m.end + 8).min(chars.len())].iter().collect::<String>());
            }
        }
    }
    println!("{:22} {:>7} {:>7} {:>7} {:>7}", "rule", "gold", "recall", "precis", "F1");
    let (mut sum_f, mut n_f) = (0.0f64, 0.0f64);
    let (mut g_all, mut f_all, mut m_all, mut r_all) = (0, 0, 0, 0);
    for (rule, [g, f, m, r]) in &tally {
        let rec = if *g > 0 { *f as f64 / *g as f64 } else { 0.0 };
        let pre = if *m > 0 { *r as f64 / *m as f64 } else { 0.0 };
        let f1 = if rec + pre > 0.0 { 2.0 * rec * pre / (rec + pre) } else { 0.0 };
        if *g > 0 {
            sum_f += f1;
            n_f += 1.0;
        }
        (g_all, f_all, m_all, r_all) = (g_all + g, f_all + f, m_all + m, r_all + r);
        println!("{rule:22} {g:7} {:6.1}% {:6.1}% {:6.1}%", 100.0 * rec, 100.0 * pre, 100.0 * f1);
    }
    let (rec, pre) = (f_all as f64 / g_all.max(1) as f64, r_all as f64 / m_all.max(1) as f64);
    println!("all annotations: recall {:.1}%, precision {:.1}%, F1 {:.1}%; mean F1 over rules {:.1}%", 100.0 * rec, 100.0 * pre, 200.0 * rec * pre / (rec + pre).max(1e-9), 100.0 * sum_f / n_f.max(1.0));
    Ok(())
}
