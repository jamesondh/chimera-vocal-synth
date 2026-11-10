//! Batch renderer for fitting the flatulence creature to recordings (tools/flatus-fit/):
//! `cargo run --release -p chimera --example flatus_render` reads one job per line on stdin,
//! `<out.f32> <seconds> <param>=<value> … [pushes=<on>:<off>,<on>:<off>…]`, renders one event
//! of the held voice at 48 kHz with those controls: the gate opens at 0 and stays open (one
//! pocket of gas, no refill), and the abdominal push is the given `push` during the listed
//! intervals (seconds; default the whole render) and 0 between them, as the staccato call does.
//!
//! A control may instead be given a course, `<param>=<t>:<v>,<t>:<v>,…` (seconds, values; linear
//! between the keys, held before the first and after the last), set every 1 ms, as a call's
//! keyframes move it. For the external sphincter's command (`eas`) the creature passes the course
//! through the muscle's activation dynamics, so the sphincter follows it no faster than muscle can.
//!
//! The render stops early once no push is still to start and the output has stayed 60 dB below
//! its loudest 0.1 s for 0.3 s (the event is over), or after 1.5 s below 1e-5. It writes raw
//! little-endian f32 to the file and prints `ok <path>`. Only the creature's own physical controls
//! can be set.
use chimera::creatures::{CreatureVoice, creature_by_id};
use std::io::{BufRead, Write};

/// A control's course: keys (t, v), linear between them.
fn course_at(keys: &[(f64, f64)], t: f64) -> f64 {
    if t <= keys[0].0 {
        return keys[0].1;
    }
    for w in keys.windows(2) {
        if t < w[1].0 {
            let u = (t - w[0].0) / (w[1].0 - w[0].0).max(1e-9);
            return w[0].1 + (w[1].1 - w[0].1) * u;
        }
    }
    keys[keys.len() - 1].1
}

fn main() {
    let def = creature_by_id("flatulence").unwrap();
    let fs = 48000.0;
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut it = line.split_whitespace();
        let Some(path) = it.next() else { continue };
        let secs: f64 = it.next().expect("seconds").parse().expect("seconds");
        let mut init = Vec::new();
        let mut courses: Vec<(usize, Vec<(f64, f64)>)> = Vec::new();
        let mut pushes: Vec<(f64, f64)> = vec![(0.0, secs)];
        let mut push = def.params[def.param_index("push").unwrap()].default;
        for a in it {
            let (k, v) = a.split_once('=').expect("param=value");
            if k == "pushes" {
                pushes = v
                    .split(',')
                    .map(|g| {
                        let (a, b) = g.split_once(':').expect("on:off");
                        (a.parse().unwrap(), b.parse().unwrap())
                    })
                    .collect();
            } else if k == "push" {
                push = v.parse().unwrap();
            } else {
                let i = def.param_index(k).unwrap_or_else(|| panic!("no parameter {k}"));
                if v.contains(':') {
                    let keys: Vec<(f64, f64)> = v
                        .split(',')
                        .map(|g| {
                            let (a, b) = g.split_once(':').expect("t:v");
                            (a.parse().unwrap(), b.parse().unwrap())
                        })
                        .collect();
                    init.push((i, keys[0].1));
                    courses.push((i, keys));
                } else {
                    init.push((i, v.parse::<f64>().unwrap()));
                }
            }
        }
        let ip = def.param_index("push").unwrap();
        let on = |t: f64| pushes.iter().any(|&(a, b)| t >= a && t < b);
        init.push((ip, if on(0.0) { push } else { 0.0 }));
        let mut v = CreatureVoice::new(def, fs, &init);
        v.set_gate(1.0);
        let n = (secs * fs) as usize;
        let mut out = vec![0.0f32; n];
        // render in blocks that end at every push edge (and every 1 ms while a course runs)
        let mut edges: Vec<(usize, f64)> = Vec::new();
        for &(a, b) in &pushes {
            edges.push(((a * fs) as usize, push));
            edges.push(((b * fs) as usize, 0.0));
        }
        edges.sort_by(|a, b| a.0.cmp(&b.0));
        let course_end = courses.iter().map(|(_, k)| k[k.len() - 1].0).fold(0.0, f64::max);
        let last_start = pushes.iter().map(|p| p.0).fold(course_end, f64::max);
        let tick = (0.001 * fs) as usize;
        let win = (0.1 * fs) as usize;
        let (mut peak, mut quiet_for) = (0.0f64, 0usize);
        let mut i = 0;
        let mut e = 0;
        while i < n {
            if i % win == 0 && i >= win {
                let r = (out[i - win..i].iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / win as f64).sqrt();
                peak = peak.max(r);
                quiet_for = if r <= peak * 1e-3 { quiet_for + win } else { 0 };
                let t = i as f64 / fs;
                let silent = peak < 1e-5 && t >= 1.5;
                if t > last_start && (silent || (peak >= 1e-5 && quiet_for as f64 / fs >= 0.3)) {
                    break;
                }
            }
            while e < edges.len() && edges[e].0 <= i {
                v.set_params(&[(ip, edges[e].1)]);
                e += 1;
            }
            let t = i as f64 / fs;
            for (k, keys) in &courses {
                v.set_params(&[(*k, course_at(keys, t))]);
            }
            let mut next = if e < edges.len() { edges[e].0.min(n) } else { n };
            if !courses.is_empty() {
                next = next.min((i / tick + 1) * tick);
            }
            let end = next.min(i + 512).min((i / win + 1) * win).max(i + 1).min(n);
            v.render(&mut out[i..end]);
            i = end;
        }
        out.truncate(i);
        let bytes: Vec<u8> = out.iter().flat_map(|x| x.to_le_bytes()).collect();
        std::fs::write(path, bytes).unwrap();
        writeln!(stdout, "ok {path}").unwrap();
        stdout.flush().unwrap();
    }
}
